use crate::metadata;
use serde_json::Value;
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};

const DISK_SIZE: u64 = 100 * 1024 * 1024;
const BLOCK_SIZE: u64 = 4 * 1024;
const TOTAL_BLOCKS: u64 = (DISK_SIZE) / (BLOCK_SIZE);

#[derive(Debug)]
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
        let utf8_string = String::from_utf8(buffer.clone()).unwrap();
        println!("{:?}", utf8_string);
        let json_result: Result<Value, _> = serde_json::from_str(&utf8_string);
        println!("{:?}", json_result);
        Ok(buffer)
    }

    pub fn write_blocks(&mut self, block_number: u64, data: &[u8]) -> io::Result<()> {
        self.file.seek(SeekFrom::Start(block_number * BLOCK_SIZE))?;
        self.file.write_all(data)?;
        Ok(())
    }

    pub fn initialize_root_dir(&mut self) -> io::Result<()> {
        let metadata = metadata::FileMetadata::new(
            0,
            String::from("/"),
            metadata::FileType::FileTypeDirectory,
            metadata::Permissions::new(true, true, true),
            0,
            0,
            vec![1],
        );
        let serialized_data = metadata::FileMetadata::serialize(&metadata);
        let _ = &self.write_blocks(0, &serialized_data)?;
        Ok(())
    }

    // pub fn allocate_metadata(&mut self, metadata: &metadata::FileMetadata) -> io::Result<u64> {
    //     let mut block_number = 0;
    //     let mut found = false;
    //     let mut buffer = vec![0; BLOCK_SIZE as usize];
    //     while block_number < TOTAL_BLOCKS {
    //         self.file.seek(SeekFrom::Start(block_number * BLOCK_SIZE))?;
    //         self.file.read_exact(&mut buffer)?;
    //         let metadata: metadata::FileMetadata = serde_json::from_slice(&buffer).unwrap();
    //         if metadata.id == 0 {
    //             found = true;
    //             break;
    //         }
    //         block_number += 1;
    //     }
    //
    //     if found {
    //         let serialized_data = metadata::FileMetadata::serialize(metadata);
    //         self.write_blocks(block_number, &serialized_data)?;
    //         Ok(block_number)
    //     } else {
    //         Err(io::Error::new(io::ErrorKind::Other, "No space left"))
    //     }
    // }
}
