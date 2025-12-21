use std::io;
use thiserror::Error;

/// Custom error type for file system operations
#[derive(Error, Debug)]
pub enum FsError {
    /// I/O error occurred during disk operations
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// Block not found or invalid block number
    #[error("Block {0} not found or invalid")]
    BlockNotFound(u64),

    /// File not found at the specified path
    #[error("File not found: {0}")]
    FileNotFound(String),

    /// Directory not found at the specified path
    #[error("Directory not found: {0}")]
    DirectoryNotFound(String),

    /// Attempted to delete a non-empty directory
    #[error("Directory not empty: {0}")]
    DirectoryNotEmpty(String),

    /// Disk is full, no more blocks available
    #[error("Disk is full - no free blocks available")]
    DiskFull,

    /// Not enough contiguous space for allocation
    #[error("Not enough contiguous space - requested {0} blocks")]
    NotEnoughContiguousSpace(u64),

    /// Invalid path format
    #[error("Invalid path: {0}")]
    InvalidPath(String),

    /// Permission denied for the operation
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// File or directory already exists
    #[error("Already exists: {0}")]
    AlreadyExists(String),

    /// Invalid file name (too long, contains invalid characters, etc.)
    #[error("Invalid file name: {0}")]
    InvalidFileName(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Deserialization error
    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    /// Invalid metadata
    #[error("Invalid metadata: {0}")]
    InvalidMetadata(String),

    /// Block is already in use
    #[error("Block {0} is already in use")]
    BlockInUse(u64),

    /// Block is already free
    #[error("Block {0} is already free")]
    BlockAlreadyFree(u64),

    /// Invalid block size
    #[error("Invalid block size: expected {expected}, got {actual}")]
    InvalidBlockSize { expected: u64, actual: u64 },

    /// Corrupted file system
    #[error("Corrupted file system: {0}")]
    CorruptedFileSystem(String),

    /// Not a directory
    #[error("Not a directory: {0}")]
    NotADirectory(String),

    /// Not a file
    #[error("Not a file: {0}")]
    NotAFile(String),

    /// Invalid offset or size for read/write operation
    #[error("Invalid offset or size: offset={offset}, size={size}")]
    InvalidOffsetOrSize { offset: u64, size: u64 },

    /// Operation not supported
    #[error("Operation not supported: {0}")]
    NotSupported(String),
}

/// Result type alias for file system operations
pub type FsResult<T> = Result<T, FsError>;

impl From<serde_json::Error> for FsError {
    fn from(err: serde_json::Error) -> Self {
        FsError::SerializationError(err.to_string())
    }
}

impl From<std::string::FromUtf8Error> for FsError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        FsError::DeserializationError(err.to_string())
    }
}