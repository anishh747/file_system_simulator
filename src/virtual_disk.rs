use crate::{
    bitmap::BlockBitmap, 
    error::FsResult, 
    serialization::{Inode, DirectoryEntry, FileType, Permissions, INODE_SIZE},
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

    pub fn read_block_metadata(&mut self, block_number: u64) -> FsResult<Vec<u8>> {
        let mut buffer = vec![0; 25 as usize];
        self.file.seek(SeekFrom::Start(block_number * BLOCK_SIZE))?;
        self.file.read_exact(&mut buffer)?;
        let utf8_string = String::from_utf8(buffer.clone())?;
        println!("{:?}", utf8_string);
        Ok(buffer)
    }

    pub fn write_block_metadata(&mut self, block_number: u64, data: &[u8]) -> FsResult<()> {
        self.file.seek(SeekFrom::Start(block_number * BLOCK_SIZE))?;
        self.file.write_all(data)?;
        Ok(())
    }

    pub fn read_blocks(&mut self, block_number: u64) -> FsResult<Vec<u8>> {
        let mut buffer = vec![0; (BLOCK_SIZE - 25) as usize];
        self.file
            .seek(SeekFrom::Start(block_number * BLOCK_SIZE + 25))?;
        self.file.read_exact(&mut buffer)?;
        let utf8_string = String::from_utf8(buffer.clone())?;
        println!("{:?}", utf8_string);
        // let json_result: Result<Value, _> = serde_json::from_str(&utf8_string);
        // println!("{:?}", json_result);
        Ok(buffer)
    }

    pub fn write_blocks(&mut self, block_number: u64, data: &[u8]) -> FsResult<()> {
        self.file
            .seek(SeekFrom::Start(block_number * BLOCK_SIZE + 25))?;
        self.file.write_all(data)?;
        Ok(())
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

    /// Allocate a single free block
    /// 
    /// Returns the block number if successful, or an error if disk is full
    pub fn allocate_block(&mut self) -> FsResult<u64> {
        let block = self.bitmap.allocate_block()?;
        self.bitmap.save(&mut self.file, BLOCK_SIZE)?;
        Ok(block)
    }

    /// Allocate multiple contiguous blocks
    /// 
    /// This is more efficient for large files as it reduces fragmentation.
    /// Returns the starting block number if successful.
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
