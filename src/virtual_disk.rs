use std::fs::{File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};

const DISK_SIZE: u64 = 100 * 1024 * 1024;
const BLOCK_SIZE: u64 = 4 * 1024;
const TOTAL_BLOCKS: u64 = (DISK_SIZE) / (BLOCK_SIZE);

pub struct VirtualDisk {
    file: File,
}

impl VirtualDisk {
    pub fn new(path: &str) -> io::Result<VirtualDisk> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;
        file.set_len(DISK_SIZE)?;

        Ok(VirtualDisk { file })
    }

    pub fn read_blocks(&mut self, block_number: u64) -> io::Result<Vec<u8>> {
        let mut buffer = vec![0; BLOCK_SIZE as usize];
        self.file.seek(SeekFrom::Start(block_number * BLOCK_SIZE))?;
        self.file.read_exact(&mut buffer)?;
        Ok(buffer)
    }

    pub fn write_blocks(&mut self, block_number: u64, data: &[u8]) -> io::Result<()> {
        self.file.seek(SeekFrom::Start(block_number * BLOCK_SIZE))?;
        self.file.write_all(data)?;
        Ok(())
    }

    pub fn allocate_blocks(&mut self, num_blocks: u64) -> io::Result<()> {
        Ok(())
    }
}
