use file_system_simulator::virtual_disk::VirtualDisk;
use file_system_simulator::serialization::Permissions;

fn main() {
    println!("=== File System Simulator - File Operations Demo ===\n");
    
    let mut disk = VirtualDisk::new("/Users/anishtimalsina/Desktop/projects/file_system_simulator/test.txt").unwrap();
    
    // Display initial disk statistics
    println!("Initial Disk Statistics:");
    println!("  Total blocks: {}", disk.total_blocks());
    println!("  Free blocks: {}", disk.free_blocks_count());
    println!("  Utilization: {:.2}%\n", disk.utilization());
    
    // Create a file
    println!("Creating file...");
    let perms = Permissions::new(true, true, false);
    let file_inode_block = disk.create_file(1, perms).unwrap();
    println!("  File created with inode at block {}\n", file_inode_block);
    
    // Write data to the file
    println!("Writing data to file...");
    let data = b"Hello, File System! This is a test file with some content.";
    disk.write_file(file_inode_block, data).unwrap();
    println!("  Written {} bytes\n", data.len());
    
    // Read the file info
    println!("Reading file info...");
    let info = disk.get_file_info(file_inode_block).unwrap();
    println!("  Inode number: {}", info.inode_number);
    println!("  File type: {:?}", info.file_type);
    println!("  Size: {} bytes", info.size);
    println!("  Block count: {}", info.block_count);
    println!("  Permissions: r={}, w={}, x={}\n", 
             perms.read(), perms.write(), perms.execute());
    
    // Read the file back
    println!("Reading file contents...");
    let read_data = disk.read_file(file_inode_block).unwrap();
    let content = String::from_utf8(read_data.clone()).unwrap();
    println!("  Read {} bytes", read_data.len());
    println!("  Content: \"{}\"\n", content);
    
    // Verify data integrity
    assert_eq!(data.to_vec(), read_data);
    println!("  ✓ Data integrity verified\n");
    
    // Test with larger file (multi-block)
    println!("Testing multi-block file...");
    let large_data: Vec<u8> = (0..10000).map(|i| (i % 256) as u8).collect();
    disk.write_file(file_inode_block, &large_data).unwrap();
    println!("  Written {} bytes", large_data.len());
    
    let info = disk.get_file_info(file_inode_block).unwrap();
    println!("  File now uses {} blocks\n", info.block_count);
    
    // Read large file back
    let read_large = disk.read_file(file_inode_block).unwrap();
    assert_eq!(large_data, read_large);
    println!("  ✓ Large file read successfully\n");
    
    // Display disk statistics after operations
    println!("Disk Statistics After File Operations:");
    println!("  Used blocks: {}", disk.used_blocks_count());
    println!("  Free blocks: {}", disk.free_blocks_count());
    println!("  Utilization: {:.2}%\n", disk.utilization());
    
    // Delete the file
    println!("Deleting file...");
    disk.delete_file(file_inode_block).unwrap();
    println!("  File deleted\n");
    
    // Final statistics
    println!("Final Disk Statistics:");
    println!("  Used blocks: {}", disk.used_blocks_count());
    println!("  Free blocks: {}", disk.free_blocks_count());
    println!("  Utilization: {:.2}%", disk.utilization());
    
    println!("\n✓ File operations test completed successfully!");
}
