use crate::{
    bitmap::BlockBitmap, 
    error::{FsError, FsResult}, 
    serialization::{Inode, DirectoryEntry, FileType, Permissions, INODE_SIZE, DIRECT_POINTERS},
};
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};

const DISK_SIZE: u64 = 100 * 1024 * 1024;
const BLOCK_SIZE: u64 = 4 * 1024;
const TOTAL_BLOCKS: u64 = (DISK_SIZE) / (BLOCK_SIZE);

#[derive(Debug)]
pub struct VirtualDisk {
    file: File,
    bitmap: BlockBitmap,
}

impl VirtualDisk {
    pub fn new(path: &str) -> FsResult<VirtualDisk> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;
        
        let file_metadata = file.metadata()?;
        let is_new_disk = file_metadata.len() == 0;
        
        file.set_len(DISK_SIZE)?;

        let bitmap = if is_new_disk {
            // Create new bitmap for fresh disk
            let bitmap = BlockBitmap::new(TOTAL_BLOCKS, BLOCK_SIZE);
            bitmap.save(&mut file, BLOCK_SIZE)?;
            bitmap
        } else {
            // Load existing bitmap from disk
            BlockBitmap::load(&mut file, TOTAL_BLOCKS, BLOCK_SIZE)?
        };

        Ok(VirtualDisk { file, bitmap })
    }

    pub fn initialize_root_dir(&mut self) -> FsResult<()> {
        // Allocate a block for root directory inode
        let root_block = self.allocate_block()?;
        
        // Create root directory inode (inode 0)
        let perms = Permissions::new(true, true, true);
        let root_inode = Inode::new(0, FileType::Directory, perms);
        
        // Write root inode to disk
        self.write_inode(root_block, &root_inode)?;
        
        Ok(())
    }

    /// Write an inode to a specific block
    pub fn write_inode(&mut self, block_number: u64, inode: &Inode) -> FsResult<()> {
        let bytes = inode.to_bytes();
        self.file.seek(SeekFrom::Start(block_number * BLOCK_SIZE))?;
        self.file.write_all(&bytes)?;
        self.file.flush()?;
        Ok(())
    }

    /// Read an inode from a specific block
    pub fn read_inode(&mut self, block_number: u64) -> FsResult<Inode> {
        let mut buffer = [0u8; INODE_SIZE];
        self.file.seek(SeekFrom::Start(block_number * BLOCK_SIZE))?;
        self.file.read_exact(&mut buffer)?;
        Inode::from_bytes(&buffer)
    }

    /// Write a directory entry to a specific offset in a block
    pub fn write_dir_entry(
        &mut self,
        block_number: u64,
        entry_index: usize,
        entry: &DirectoryEntry,
    ) -> FsResult<()> {
        let bytes = entry.to_bytes();
        let offset = block_number * BLOCK_SIZE + (entry_index * DirectoryEntry::ENTRY_SIZE) as u64;
        self.file.seek(SeekFrom::Start(offset))?;
        self.file.write_all(&bytes)?;
        self.file.flush()?;
        Ok(())
    }

    /// Read a directory entry from a specific offset in a block
    pub fn read_dir_entry(
        &mut self,
        block_number: u64,
        entry_index: usize,
    ) -> FsResult<DirectoryEntry> {
        let mut buffer = [0u8; DirectoryEntry::ENTRY_SIZE];
        let offset = block_number * BLOCK_SIZE + (entry_index * DirectoryEntry::ENTRY_SIZE) as u64;
        self.file.seek(SeekFrom::Start(offset))?;
        self.file.read_exact(&mut buffer)?;
        DirectoryEntry::from_bytes(&buffer)
    }

    // ==================== FILE OPERATIONS ====================

    /// Create a new file and return its inode block number
    /// 
    /// This allocates an inode block and initializes it with file metadata
    pub fn create_file(
        &mut self,
        inode_number: u64,
        permissions: Permissions,
    ) -> FsResult<u64> {
        // Allocate a block for the inode
        let inode_block = self.allocate_block()?;
        
        // Create the inode
        let inode = Inode::new(inode_number, FileType::File, permissions);
        
        // Write inode to disk
        self.write_inode(inode_block, &inode)?;
        
        Ok(inode_block)
    }

    /// Write data to a file
    /// 
    /// This handles multi-block files by allocating blocks as needed
    /// and updating the inode's block pointers
    pub fn write_file(
        &mut self,
        inode_block: u64,
        data: &[u8],
    ) -> FsResult<()> {
        // Read the current inode
        let mut inode = self.read_inode(inode_block)?;
        
        // Verify it's a file
        if inode.file_type != FileType::File {
            return Err(FsError::NotAFile(format!("Inode {} is not a file", inode.inode_number)));
        }
        
        // Calculate how many blocks we need
        let blocks_needed = ((data.len() as u64 + BLOCK_SIZE - 1) / BLOCK_SIZE) as usize;
        
        if blocks_needed > DIRECT_POINTERS {
            return Err(FsError::NotSupported(
                format!("File size {} bytes requires {} blocks, but only {} direct pointers supported", 
                        data.len(), blocks_needed, DIRECT_POINTERS)
            ));
        }
        
        // Free old blocks if they exist
        for i in 0..inode.block_count as usize {
            if inode.direct_blocks[i] != 0 {
                self.free_block(inode.direct_blocks[i])?;
                inode.direct_blocks[i] = 0;
            }
        }
        
        // Allocate new blocks and write data
        let mut offset = 0;
        for i in 0..blocks_needed {
            let block = self.allocate_block()?;
            inode.direct_blocks[i] = block;
            
            // Calculate how much data to write to this block
            let remaining = data.len() - offset;
            let to_write = remaining.min(BLOCK_SIZE as usize);
            
            // Write data to block
            self.file.seek(SeekFrom::Start(block * BLOCK_SIZE))?;
            self.file.write_all(&data[offset..offset + to_write])?;
            
            offset += to_write;
        }
        
        // Update inode metadata
        inode.size = data.len() as u64;
        inode.block_count = blocks_needed as u64;
        inode.modified = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // Write updated inode back to disk
        self.write_inode(inode_block, &inode)?;
        self.file.flush()?;
        
        Ok(())
    }

    /// Read data from a file
    /// 
    /// Reads the entire file contents by following the inode's block pointers
    pub fn read_file(&mut self, inode_block: u64) -> FsResult<Vec<u8>> {
        // Read the inode
        let inode = self.read_inode(inode_block)?;
        
        // Verify it's a file
        if inode.file_type != FileType::File {
            return Err(FsError::NotAFile(format!("Inode {} is not a file", inode.inode_number)));
        }
        
        // Allocate buffer for file data
        let mut data = Vec::with_capacity(inode.size as usize);
        
        // Read each block
        let mut remaining = inode.size;
        for i in 0..inode.block_count as usize {
            let block = inode.direct_blocks[i];
            if block == 0 {
                return Err(FsError::CorruptedFileSystem(
                    format!("Inode {} has null block pointer at index {}", inode.inode_number, i)
                ));
            }
            
            // Read block data
            let to_read = remaining.min(BLOCK_SIZE);
            let mut buffer = vec![0u8; to_read as usize];
            
            self.file.seek(SeekFrom::Start(block * BLOCK_SIZE))?;
            self.file.read_exact(&mut buffer)?;
            
            data.extend_from_slice(&buffer);
            remaining -= to_read;
        }
        
        Ok(data)
    }

    /// Delete a file
    /// 
    /// Frees all blocks used by the file including the inode block
    pub fn delete_file(&mut self, inode_block: u64) -> FsResult<()> {
        // Read the inode
        let inode = self.read_inode(inode_block)?;
        
        // Verify it's a file
        if inode.file_type != FileType::File {
            return Err(FsError::NotAFile(format!("Inode {} is not a file", inode.inode_number)));
        }
        
        // Free all data blocks
        for i in 0..inode.block_count as usize {
            if inode.direct_blocks[i] != 0 {
                self.free_block(inode.direct_blocks[i])?;
            }
        }
        
        // Free the inode block itself
        self.free_block(inode_block)?;
        
        Ok(())
    }

    /// Get file information
    pub fn get_file_info(&mut self, inode_block: u64) -> FsResult<Inode> {
        let inode = self.read_inode(inode_block)?;
        
        if inode.file_type != FileType::File {
            return Err(FsError::NotAFile(format!("Inode {} is not a file", inode.inode_number)));
        }
        
        Ok(inode)
    }

    // ==================== DIRECTORY OPERATIONS ====================

    /// Create a new directory and return its inode block number
    pub fn create_directory(
        &mut self,
        inode_number: u64,
        permissions: Permissions,
    ) -> FsResult<u64> {
        // Allocate a block for the directory inode
        let inode_block = self.allocate_block()?;
        
        // Allocate a block for directory entries
        let entries_block = self.allocate_block()?;
        
        // Create the directory inode
        let mut inode = Inode::new(inode_number, FileType::Directory, permissions);
        inode.direct_blocks[0] = entries_block;
        inode.block_count = 1;
        
        // Write inode to disk
        self.write_inode(inode_block, &inode)?;
        
        Ok(inode_block)
    }

    /// Add an entry to a directory
    pub fn add_directory_entry(
        &mut self,
        dir_inode_block: u64,
        entry: DirectoryEntry,
    ) -> FsResult<()> {
        // Read the directory inode
        let inode = self.read_inode(dir_inode_block)?;
        
        // Verify it's a directory
        if inode.file_type != FileType::Directory {
            return Err(FsError::NotADirectory(format!("Inode {} is not a directory", inode.inode_number)));
        }
        
        // Get the directory entries block
        let entries_block = inode.direct_blocks[0];
        if entries_block == 0 {
            return Err(FsError::CorruptedFileSystem("Directory has no entries block".to_string()));
        }
        
        // Calculate how many entries fit in a block
        let entries_per_block = (BLOCK_SIZE as usize) / DirectoryEntry::ENTRY_SIZE;
        
        // Find first empty slot
        for i in 0..entries_per_block {
            // Try to read existing entry
            match self.read_dir_entry(entries_block, i) {
                Ok(_) => continue, // Slot occupied
                Err(FsError::InvalidMetadata(_)) => {
                    // Empty slot found, write new entry
                    self.write_dir_entry(entries_block, i, &entry)?;
                    return Ok(());
                }
                Err(e) => return Err(e),
            }
        }
        
        Err(FsError::NotSupported("Directory is full".to_string()))
    }

    /// Remove an entry from a directory by name
    pub fn remove_directory_entry(
        &mut self,
        dir_inode_block: u64,
        name: &str,
    ) -> FsResult<u64> {
        // Read the directory inode
        let inode = self.read_inode(dir_inode_block)?;
        
        // Verify it's a directory
        if inode.file_type != FileType::Directory {
            return Err(FsError::NotADirectory(format!("Inode {} is not a directory", inode.inode_number)));
        }
        
        // Get the directory entries block
        let entries_block = inode.direct_blocks[0];
        if entries_block == 0 {
            return Err(FsError::CorruptedFileSystem("Directory has no entries block".to_string()));
        }
        
        // Calculate how many entries fit in a block
        let entries_per_block = (BLOCK_SIZE as usize) / DirectoryEntry::ENTRY_SIZE;
        
        // Find and remove the entry
        for i in 0..entries_per_block {
            match self.read_dir_entry(entries_block, i) {
                Ok(entry) => {
                    if entry.name == name {
                        // Found it! Clear the entry by writing zeros
                        let empty_entry = [0u8; DirectoryEntry::ENTRY_SIZE];
                        let offset = entries_block * BLOCK_SIZE + (i * DirectoryEntry::ENTRY_SIZE) as u64;
                        self.file.seek(SeekFrom::Start(offset))?;
                        self.file.write_all(&empty_entry)?;
                        self.file.flush()?;
                        return Ok(entry.inode_number);
                    }
                }
                Err(FsError::InvalidMetadata(_)) => continue, // Empty slot
                Err(e) => return Err(e),
            }
        }
        
        Err(FsError::FileNotFound(name.to_string()))
    }

    /// List all entries in a directory
    pub fn list_directory(&mut self, dir_inode_block: u64) -> FsResult<Vec<DirectoryEntry>> {
        // Read the directory inode
        let inode = self.read_inode(dir_inode_block)?;
        
        // Verify it's a directory
        if inode.file_type != FileType::Directory {
            return Err(FsError::NotADirectory(format!("Inode {} is not a directory", inode.inode_number)));
        }
        
        // Get the directory entries block
        let entries_block = inode.direct_blocks[0];
        if entries_block == 0 {
            return Err(FsError::CorruptedFileSystem("Directory has no entries block".to_string()));
        }
        
        // Calculate how many entries fit in a block
        let entries_per_block = (BLOCK_SIZE as usize) / DirectoryEntry::ENTRY_SIZE;
        
        // Collect all valid entries
        let mut entries = Vec::new();
        for i in 0..entries_per_block {
            match self.read_dir_entry(entries_block, i) {
                Ok(entry) => entries.push(entry),
                Err(FsError::InvalidMetadata(_)) => continue, // Empty slot
                Err(e) => return Err(e),
            }
        }
        
        Ok(entries)
    }

    /// Find an entry in a directory by name
    pub fn find_directory_entry(
        &mut self,
        dir_inode_block: u64,
        name: &str,
    ) -> FsResult<DirectoryEntry> {
        let entries = self.list_directory(dir_inode_block)?;
        
        for entry in entries {
            if entry.name == name {
                return Ok(entry);
            }
        }
        
        Err(FsError::FileNotFound(name.to_string()))
    }

    /// Delete a directory (must be empty)
    pub fn delete_directory(&mut self, dir_inode_block: u64) -> FsResult<()> {
        // Read the directory inode
        let inode = self.read_inode(dir_inode_block)?;
        
        // Verify it's a directory
        if inode.file_type != FileType::Directory {
            return Err(FsError::NotADirectory(format!("Inode {} is not a directory", inode.inode_number)));
        }
        
        // Check if directory is empty
        let entries = self.list_directory(dir_inode_block)?;
        if !entries.is_empty() {
            return Err(FsError::DirectoryNotEmpty(format!("Directory has {} entries", entries.len())));
        }
        
        // Free the entries block
        if inode.direct_blocks[0] != 0 {
            self.free_block(inode.direct_blocks[0])?;
        }
        
        // Free the inode block
        self.free_block(dir_inode_block)?;
        
        Ok(())
    }

    /// Get directory information
    pub fn get_directory_info(&mut self, dir_inode_block: u64) -> FsResult<Inode> {
        let inode = self.read_inode(dir_inode_block)?;
        
        if inode.file_type != FileType::Directory {
            return Err(FsError::NotADirectory(format!("Inode {} is not a directory", inode.inode_number)));
        }
        
        Ok(inode)
    }

    // ==================== BLOCK ALLOCATION ====================

    /// Allocate a single free block
    pub fn allocate_block(&mut self) -> FsResult<u64> {
        let block = self.bitmap.allocate_block()?;
        self.bitmap.save(&mut self.file, BLOCK_SIZE)?;
        Ok(block)
    }

    /// Allocate multiple contiguous blocks
    pub fn allocate_contiguous_blocks(&mut self, count: u64) -> FsResult<u64> {
        let start = self.bitmap.allocate_contiguous(count)?;
        self.bitmap.save(&mut self.file, BLOCK_SIZE)?;
        Ok(start)
    }

    /// Free a previously allocated block
    pub fn free_block(&mut self, block: u64) -> FsResult<()> {
        self.bitmap.free_block(block);
        self.bitmap.save(&mut self.file, BLOCK_SIZE)?;
        Ok(())
    }

    /// Free multiple contiguous blocks
    pub fn free_blocks(&mut self, start: u64, count: u64) -> FsResult<()> {
        self.bitmap.free_blocks(start, count);
        self.bitmap.save(&mut self.file, BLOCK_SIZE)?;
        Ok(())
    }

    /// Check if a block is currently in use
    pub fn is_block_used(&self, block: u64) -> bool {
        self.bitmap.is_block_used(block)
    }

    /// Get the total number of blocks in the file system
    pub fn total_blocks(&self) -> u64 {
        self.bitmap.total_blocks()
    }

    /// Get the number of free blocks available
    pub fn free_blocks_count(&self) -> u64 {
        self.bitmap.count_free_blocks()
    }

    /// Get the number of used blocks
    pub fn used_blocks_count(&self) -> u64 {
        self.bitmap.count_used_blocks()
    }

    /// Get disk utilization as a percentage (0.0 to 100.0)
    pub fn utilization(&self) -> f64 {
        self.bitmap.utilization()
    }

    /// Save the current bitmap state to disk
    pub fn sync_bitmap(&mut self) -> FsResult<()> {
        self.bitmap.save(&mut self.file, BLOCK_SIZE)
    }
}
