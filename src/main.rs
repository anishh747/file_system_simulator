use file_system_simulator::virtual_disk::VirtualDisk;
use file_system_simulator::serialization::{Inode, DirectoryEntry, FileType, Permissions};

fn main() {
    println!("=== File System Simulator - Binary Serialization Demo ===\n");
    
    let mut disk = VirtualDisk::new("/Users/anishtimalsina/Desktop/projects/file_system_simulator/test.txt").unwrap();
    
    // Display disk statistics
    println!("Disk Statistics:");
    println!("  Total blocks: {}", disk.total_blocks());
    println!("  Used blocks: {}", disk.used_blocks_count());
    println!("  Free blocks: {}", disk.free_blocks_count());
    println!("  Utilization: {:.2}%\n", disk.utilization());
    
    // Initialize root directory
    println!("Initializing root directory...");
    disk.initialize_root_dir().unwrap();
    println!("  Root directory created\n");
    
    // Create and write a test inode
    println!("Creating test file inode...");
    let perms = Permissions::new(true, true, false);
    let mut file_inode = Inode::new(1, FileType::File, perms);
    file_inode.size = 1024;
    file_inode.block_count = 1;
    
    let file_block = disk.allocate_block().unwrap();
    disk.write_inode(file_block, &file_inode).unwrap();
    println!("  File inode written to block {}", file_block);
    println!("  Inode number: {}", file_inode.inode_number);
    println!("  File type: {:?}", file_inode.file_type);
    println!("  Size: {} bytes", file_inode.size);
    println!("  Permissions: r={}, w={}, x={}\n", 
             perms.read(), perms.write(), perms.execute());
    
    // Read the inode back
    println!("Reading inode back from disk...");
    let read_inode = disk.read_inode(file_block).unwrap();
    println!("  Inode number: {}", read_inode.inode_number);
    println!("  File type: {:?}", read_inode.file_type);
    println!("  Size: {} bytes", read_inode.size);
    println!("  Block count: {}", read_inode.block_count);
    
    // Verify data integrity
    assert_eq!(file_inode.inode_number, read_inode.inode_number);
    assert_eq!(file_inode.size, read_inode.size);
    println!("  ✓ Data integrity verified\n");
    
    // Create directory entries
    println!("Creating directory entries...");
    let dir_block = disk.allocate_block().unwrap();
    
    let entry1 = DirectoryEntry::new(1, FileType::File, "test.txt".to_string()).unwrap();
    let entry2 = DirectoryEntry::new(2, FileType::Directory, "documents".to_string()).unwrap();
    let entry3 = DirectoryEntry::new(3, FileType::File, "readme.md".to_string()).unwrap();
    
    disk.write_dir_entry(dir_block, 0, &entry1).unwrap();
    disk.write_dir_entry(dir_block, 1, &entry2).unwrap();
    disk.write_dir_entry(dir_block, 2, &entry3).unwrap();
    
    println!("  Written 3 directory entries to block {}", dir_block);
    
    // Read directory entries back
    println!("\nReading directory entries...");
    for i in 0..3 {
        let entry = disk.read_dir_entry(dir_block, i).unwrap();
        println!("  Entry {}: inode={}, type={:?}, name='{}'", 
                 i, entry.inode_number, entry.file_type, entry.name);
    }
    
    // Final statistics
    println!("\nFinal Disk Statistics:");
    println!("  Used blocks: {}", disk.used_blocks_count());
    println!("  Free blocks: {}", disk.free_blocks_count());
    println!("  Utilization: {:.2}%", disk.utilization());
    
    println!("\n✓ Binary serialization test completed successfully!");
}
