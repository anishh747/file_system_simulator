use std::time::SystemTime;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FileType {
    FileTypeFile,
    FileTypeDirectory,
}

#[derive(Debug, Clone)]
pub struct Permissions {
    pub read: bool,
    pub write: bool,
    pub execute: bool,
}

impl Permissions {
    pub fn new(read: bool, write: bool, execute: bool) -> Permissions {
        Permissions {
            read,
            write,
            execute,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Timestamp {
    pub created: SystemTime,
    pub modified: SystemTime,
    pub accessed: SystemTime,
}

impl Timestamp {
    pub fn new() -> Timestamp {
        Timestamp {
            created: SystemTime::now(),
            modified: SystemTime::now(),
            accessed: SystemTime::now(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub id: u64,
    pub name: String,
    pub file_type: FileType,
    pub permissions: Permissions,
    pub size: u64,
    pub timestamp: Timestamp,
    pub block_count: u64,
    pub block_start: Vec<u64>,
}

impl FileMetadata {
    pub fn new(
        id: u64,
        name: String,
        file_type: FileType,
        permissions: Permissions,
        size: u64,
        block_count: u64,
        block_start: Vec<u64>,
    ) -> FileMetadata {
        FileMetadata {
            id,
            name,
            file_type,
            permissions,
            size,
            timestamp: Timestamp::new(),
            block_count,
            block_start,
        }
    }
}
