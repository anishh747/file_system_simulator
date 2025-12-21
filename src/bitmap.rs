use crate::error::{FsError, FsResult};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};

/// Bitmap-based block allocator for tracking free and used blocks
/// The bitmap is stored at the beginning of the virtual disk.
/// Each bit represents one block: 0 = free, 1 = used
#[derive(Debug)]
pub struct BlockBitmap {
    /// Total number of blocks in the file system
    total_blocks: u64,
    /// Number of blocks reserved for the bitmap itself
    bitmap_blocks: u64,
    /// In-memory bitmap representation
    bitmap: Vec<u8>,
}

impl BlockBitmap {
    /// Calculate how many blocks are needed to store the bitmap
    /// Each block can hold BLOCK_SIZE * 8 bits (one bit per block)
    pub fn calculate_bitmap_blocks(total_blocks: u64, block_size: u64) -> u64 {
        let bits_per_block = block_size * 8;
        (total_blocks + bits_per_block - 1) / bits_per_block
    }

    /// Create a new bitmap for the given number of blocks
    pub fn new(total_blocks: u64, block_size: u64) -> Self {
        let bitmap_blocks = Self::calculate_bitmap_blocks(total_blocks, block_size);
        let bitmap_bytes = ((total_blocks + 7) / 8) as usize;
        
        let mut bitmap = vec![0u8; bitmap_bytes];
        
        // Mark bitmap blocks and superblock as used
        let reserved_blocks = bitmap_blocks + 1; // +1 for superblock
        for block in 0..reserved_blocks {
            Self::set_bit(&mut bitmap, block);
        }
        
        BlockBitmap {
            total_blocks,
            bitmap_blocks,
            bitmap,
        }
    }

    /// Load bitmap from disk
    pub fn load(file: &mut File, total_blocks: u64, block_size: u64) -> FsResult<Self> {
        let bitmap_blocks = Self::calculate_bitmap_blocks(total_blocks, block_size);
        let bitmap_bytes = ((total_blocks + 7) / 8) as usize;
        
        let mut bitmap = vec![0u8; bitmap_bytes];
        
        // Bitmap starts after superblock (block 0)
        file.seek(SeekFrom::Start(block_size))?;
        file.read_exact(&mut bitmap)?;
        
        Ok(BlockBitmap {
            total_blocks,
            bitmap_blocks,
            bitmap,
        })
    }

    /// Save bitmap to disk
    pub fn save(&self, file: &mut File, block_size: u64) -> FsResult<()> {
        // Bitmap starts after superblock (block 0)
        file.seek(SeekFrom::Start(block_size))?;
        file.write_all(&self.bitmap)?;
        file.flush()?;
        Ok(())
    }

    /// Allocate a single free block
    /// Returns the block number if successful, or error if disk is full
    pub fn allocate_block(&mut self) -> FsResult<u64> {
        for block in 0..self.total_blocks {
            if !self.is_block_used(block) {
                self.mark_used(block);
                return Ok(block);
            }
        }
        Err(FsError::DiskFull)
    }

    /// Allocate multiple contiguous blocks
    /// Returns the starting block number if successful
    pub fn allocate_contiguous(&mut self, count: u64) -> FsResult<u64> {
        if count == 0 {
            return Err(FsError::InvalidOffsetOrSize { offset: 0, size: 0 });
        }

        let mut start = 0;
        let mut consecutive = 0;

        for block in 0..self.total_blocks {
            if !self.is_block_used(block) {
                if consecutive == 0 {
                    start = block;
                }
                consecutive += 1;
                
                if consecutive == count {
                    // Mark all blocks as used
                    for b in start..(start + count) {
                        self.mark_used(b);
                    }
                    return Ok(start);
                }
            } else {
                consecutive = 0;
            }
        }

        Err(FsError::NotEnoughContiguousSpace(count))
    }

    /// Free a block, making it available for allocation
    pub fn free_block(&mut self, block: u64) {
        if block < self.total_blocks {
            Self::clear_bit(&mut self.bitmap, block);
        }
    }

    /// Free multiple contiguous blocks
    pub fn free_blocks(&mut self, start: u64, count: u64) {
        for block in start..(start + count) {
            self.free_block(block);
        }
    }

    /// Check if a block is currently in use
    pub fn is_block_used(&self, block: u64) -> bool {
        if block >= self.total_blocks {
            return true; // Out of bounds blocks are considered "used"
        }
        
        let byte_index = (block / 8) as usize;
        let bit_index = (block % 8) as u8;
        
        if byte_index >= self.bitmap.len() {
            return true;
        }
        
        (self.bitmap[byte_index] & (1 << bit_index)) != 0
    }

    /// Mark a block as used
    fn mark_used(&mut self, block: u64) {
        Self::set_bit(&mut self.bitmap, block);
    }

    /// Set a bit in the bitmap (mark as used)
    fn set_bit(bitmap: &mut [u8], block: u64) {
        let byte_index = (block / 8) as usize;
        let bit_index = (block % 8) as u8;
        
        if byte_index < bitmap.len() {
            bitmap[byte_index] |= 1 << bit_index;
        }
    }

    /// Clear a bit in the bitmap (mark as free)
    fn clear_bit(bitmap: &mut [u8], block: u64) {
        let byte_index = (block / 8) as usize;
        let bit_index = (block % 8) as u8;
        
        if byte_index < bitmap.len() {
            bitmap[byte_index] &= !(1 << bit_index);
        }
    }

    /// Get the total number of blocks
    pub fn total_blocks(&self) -> u64 {
        self.total_blocks
    }

    /// Get the number of blocks used by the bitmap itself
    pub fn bitmap_blocks(&self) -> u64 {
        self.bitmap_blocks
    }

    /// Count free blocks
    pub fn count_free_blocks(&self) -> u64 {
        let mut count = 0;
        for block in 0..self.total_blocks {
            if !self.is_block_used(block) {
                count += 1;
            }
        }
        count
    }

    /// Count used blocks
    pub fn count_used_blocks(&self) -> u64 {
        self.total_blocks - self.count_free_blocks()
    }

    /// Get utilization percentage (0.0 to 100.0)
    pub fn utilization(&self) -> f64 {
        let used = self.count_used_blocks() as f64;
        let total = self.total_blocks as f64;
        (used / total) * 100.0
    }
}