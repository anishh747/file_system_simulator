use crate::error::{FsError, FsResult};
use std::time::{SystemTime, UNIX_EPOCH};

/// Maximum file name length in bytes
pub const MAX_FILENAME_LENGTH: usize = 255;

/// Size of inode structure in bytes
pub const INODE_SIZE: usize = 512;

/// Maximum number of direct block pointers in an inode
pub const DIRECT_POINTERS: usize = 12;

/// Maximum number of indirect block pointers
pub const INDIRECT_POINTERS: usize = 3;

/// Binary serialization and deserialization for file system structures
/// 
/// This module provides fixed-size binary formats for efficient storage
/// of metadata on disk, replacing the variable-length JSON serialization.

/// File type enumeration (1 byte)
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FileType {
    File = 1,
    Directory = 2,
}

impl FileType {
    pub fn from_u8(value: u8) -> FsResult<Self> {
        match value {
            1 => Ok(FileType::File),
            2 => Ok(FileType::Directory),
            _ => Err(FsError::InvalidMetadata(format!(
                "Invalid file type: {}",
                value
            ))),
        }
    }

    pub fn to_u8(self) -> u8 {
        self as u8
    }
}

/// Permissions structure (1 byte, using bit flags)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Permissions {
    flags: u8,
}

impl Permissions {
    const READ: u8 = 0b001;
    const WRITE: u8 = 0b010;
    const EXECUTE: u8 = 0b100;

    pub fn new(read: bool, write: bool, execute: bool) -> Self {
        let mut flags = 0;
        if read {
            flags |= Self::READ;
        }
        if write {
            flags |= Self::WRITE;
        }
        if execute {
            flags |= Self::EXECUTE;
        }
        Permissions { flags }
    }

    pub fn read(&self) -> bool {
        (self.flags & Self::READ) != 0
    }

    pub fn write(&self) -> bool {
        (self.flags & Self::WRITE) != 0
    }

    pub fn execute(&self) -> bool {
        (self.flags & Self::EXECUTE) != 0
    }

    pub fn from_u8(flags: u8) -> Self {
        Permissions { flags }
    }

    pub fn to_u8(self) -> u8 {
        self.flags
    }
}

/// Inode structure - fixed size metadata for files and directories
/// 
/// Layout (512 bytes total):
/// - Magic number: 4 bytes
/// - Inode number: 8 bytes
/// - File type: 1 byte
/// - Permissions: 1 byte
/// - Link count: 2 bytes
/// - File size: 8 bytes
/// - Block count: 8 bytes
/// - Created time: 8 bytes (Unix timestamp)
/// - Modified time: 8 bytes
/// - Accessed time: 8 bytes
/// - Direct pointers: 12 * 8 = 96 bytes
/// - Indirect pointers: 3 * 8 = 24 bytes
/// - Reserved: 336 bytes (for future use)
#[derive(Debug, Clone)]
pub struct Inode {
    pub inode_number: u64,
    pub file_type: FileType,
    pub permissions: Permissions,
    pub link_count: u16,
    pub size: u64,
    pub block_count: u64,
    pub created: u64,      // Unix timestamp
    pub modified: u64,     // Unix timestamp
    pub accessed: u64,     // Unix timestamp
    pub direct_blocks: [u64; DIRECT_POINTERS],
    pub indirect_blocks: [u64; INDIRECT_POINTERS],
}

impl Inode {
    const MAGIC: u32 = 0x494E4F44; // "INOD" in ASCII

    pub fn new(inode_number: u64, file_type: FileType, permissions: Permissions) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Inode {
            inode_number,
            file_type,
            permissions,
            link_count: 1,
            size: 0,
            block_count: 0,
            created: now,
            modified: now,
            accessed: now,
            direct_blocks: [0; DIRECT_POINTERS],
            indirect_blocks: [0; INDIRECT_POINTERS],
        }
    }

    /// Serialize inode to fixed-size binary format (512 bytes)
    pub fn to_bytes(&self) -> [u8; INODE_SIZE] {
        let mut bytes = [0u8; INODE_SIZE];
        let mut offset = 0;

        // Magic number
        bytes[offset..offset + 4].copy_from_slice(&Self::MAGIC.to_le_bytes());
        offset += 4;

        // Inode number
        bytes[offset..offset + 8].copy_from_slice(&self.inode_number.to_le_bytes());
        offset += 8;

        // File type
        bytes[offset] = self.file_type.to_u8();
        offset += 1;

        // Permissions
        bytes[offset] = self.permissions.to_u8();
        offset += 1;

        // Link count
        bytes[offset..offset + 2].copy_from_slice(&self.link_count.to_le_bytes());
        offset += 2;

        // File size
        bytes[offset..offset + 8].copy_from_slice(&self.size.to_le_bytes());
        offset += 8;

        // Block count
        bytes[offset..offset + 8].copy_from_slice(&self.block_count.to_le_bytes());
        offset += 8;

        // Timestamps
        bytes[offset..offset + 8].copy_from_slice(&self.created.to_le_bytes());
        offset += 8;
        bytes[offset..offset + 8].copy_from_slice(&self.modified.to_le_bytes());
        offset += 8;
        bytes[offset..offset + 8].copy_from_slice(&self.accessed.to_le_bytes());
        offset += 8;

        // Direct block pointers
        for &block in &self.direct_blocks {
            bytes[offset..offset + 8].copy_from_slice(&block.to_le_bytes());
            offset += 8;
        }

        // Indirect block pointers
        for &block in &self.indirect_blocks {
            bytes[offset..offset + 8].copy_from_slice(&block.to_le_bytes());
            offset += 8;
        }

        // Remaining bytes are reserved (already zeroed)

        bytes
    }

    /// Deserialize inode from binary format
    pub fn from_bytes(bytes: &[u8]) -> FsResult<Self> {
        if bytes.len() < INODE_SIZE {
            return Err(FsError::InvalidMetadata(format!(
                "Inode data too short: {} bytes",
                bytes.len()
            )));
        }

        let mut offset = 0;

        // Verify magic number
        let magic = u32::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]);
        if magic != Self::MAGIC {
            return Err(FsError::CorruptedFileSystem(format!(
                "Invalid inode magic number: 0x{:08X}",
                magic
            )));
        }
        offset += 4;

        // Inode number
        let inode_number = u64::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
            bytes[offset + 4],
            bytes[offset + 5],
            bytes[offset + 6],
            bytes[offset + 7],
        ]);
        offset += 8;

        // File type
        let file_type = FileType::from_u8(bytes[offset])?;
        offset += 1;

        // Permissions
        let permissions = Permissions::from_u8(bytes[offset]);
        offset += 1;

        // Link count
        let link_count = u16::from_le_bytes([bytes[offset], bytes[offset + 1]]);
        offset += 2;

        // File size
        let size = u64::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
            bytes[offset + 4],
            bytes[offset + 5],
            bytes[offset + 6],
            bytes[offset + 7],
        ]);
        offset += 8;

        // Block count
        let block_count = u64::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
            bytes[offset + 4],
            bytes[offset + 5],
            bytes[offset + 6],
            bytes[offset + 7],
        ]);
        offset += 8;

        // Timestamps
        let created = u64::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
            bytes[offset + 4],
            bytes[offset + 5],
            bytes[offset + 6],
            bytes[offset + 7],
        ]);
        offset += 8;

        let modified = u64::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
            bytes[offset + 4],
            bytes[offset + 5],
            bytes[offset + 6],
            bytes[offset + 7],
        ]);
        offset += 8;

        let accessed = u64::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
            bytes[offset + 4],
            bytes[offset + 5],
            bytes[offset + 6],
            bytes[offset + 7],
        ]);
        offset += 8;

        // Direct block pointers
        let mut direct_blocks = [0u64; DIRECT_POINTERS];
        for block in &mut direct_blocks {
            *block = u64::from_le_bytes([
                bytes[offset],
                bytes[offset + 1],
                bytes[offset + 2],
                bytes[offset + 3],
                bytes[offset + 4],
                bytes[offset + 5],
                bytes[offset + 6],
                bytes[offset + 7],
            ]);
            offset += 8;
        }

        // Indirect block pointers
        let mut indirect_blocks = [0u64; INDIRECT_POINTERS];
        for block in &mut indirect_blocks {
            *block = u64::from_le_bytes([
                bytes[offset],
                bytes[offset + 1],
                bytes[offset + 2],
                bytes[offset + 3],
                bytes[offset + 4],
                bytes[offset + 5],
                bytes[offset + 6],
                bytes[offset + 7],
            ]);
            offset += 8;
        }

        Ok(Inode {
            inode_number,
            file_type,
            permissions,
            link_count,
            size,
            block_count,
            created,
            modified,
            accessed,
            direct_blocks,
            indirect_blocks,
        })
    }
}

/// Directory entry structure - fixed size entry in directory blocks
/// 
/// Layout (272 bytes total):
/// - Inode number: 8 bytes
/// - Entry type: 1 byte
/// - Name length: 1 byte
/// - Name: 255 bytes (null-padded)
/// - Reserved: 7 bytes
#[derive(Debug, Clone)]
pub struct DirectoryEntry {
    pub inode_number: u64,
    pub file_type: FileType,
    pub name: String,
}

impl DirectoryEntry {
    pub const ENTRY_SIZE: usize = 272;

    pub fn new(inode_number: u64, file_type: FileType, name: String) -> FsResult<Self> {
        if name.len() > MAX_FILENAME_LENGTH {
            return Err(FsError::InvalidFileName(format!(
                "Name too long: {} bytes (max {})",
                name.len(),
                MAX_FILENAME_LENGTH
            )));
        }

        Ok(DirectoryEntry {
            inode_number,
            file_type,
            name,
        })
    }

    /// Serialize directory entry to fixed-size binary format
    pub fn to_bytes(&self) -> [u8; Self::ENTRY_SIZE] {
        let mut bytes = [0u8; Self::ENTRY_SIZE];
        let mut offset = 0;

        // Inode number
        bytes[offset..offset + 8].copy_from_slice(&self.inode_number.to_le_bytes());
        offset += 8;

        // File type
        bytes[offset] = self.file_type.to_u8();
        offset += 1;

        // Name length
        let name_bytes = self.name.as_bytes();
        bytes[offset] = name_bytes.len() as u8;
        offset += 1;

        // Name (null-padded)
        bytes[offset..offset + name_bytes.len()].copy_from_slice(name_bytes);

        bytes
    }

    /// Deserialize directory entry from binary format
    pub fn from_bytes(bytes: &[u8]) -> FsResult<Self> {
        if bytes.len() < Self::ENTRY_SIZE {
            return Err(FsError::InvalidMetadata(format!(
                "Directory entry data too short: {} bytes",
                bytes.len()
            )));
        }

        let mut offset = 0;

        // Inode number
        let inode_number = u64::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
            bytes[offset + 4],
            bytes[offset + 5],
            bytes[offset + 6],
            bytes[offset + 7],
        ]);
        offset += 8;

        // Skip if inode is 0 (empty entry)
        if inode_number == 0 {
            return Err(FsError::InvalidMetadata("Empty directory entry".to_string()));
        }

        // File type
        let file_type = FileType::from_u8(bytes[offset])?;
        offset += 1;

        // Name length
        let name_len = bytes[offset] as usize;
        offset += 1;

        if name_len > MAX_FILENAME_LENGTH {
            return Err(FsError::InvalidFileName(format!(
                "Name length too long: {}",
                name_len
            )));
        }

        // Name
        let name_bytes = &bytes[offset..offset + name_len];
        let name = String::from_utf8(name_bytes.to_vec())?;

        Ok(DirectoryEntry {
            inode_number,
            file_type,
            name,
        })
    }
}