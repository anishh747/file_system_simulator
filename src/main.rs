use file_system_simulator::virtual_disk::VirtualDisk;
use file_system_simulator::serialization::{Permissions, FileType, DirectoryEntry};

fn main() {
    println!("=== File System Simulator - Directory Operations Demo ===\n");
    
    let mut disk = VirtualDisk::new("/Users/anishtimalsina/Desktop/projects/file_system_simulator/test.txt").unwrap();
    
    // Display initial disk statistics
    println!("Initial Disk Statistics:");
    println!("  Free blocks: {}", disk.free_blocks_count());
    println!("  Utilization: {:.2}%\n", disk.utilization());
    
    // Create root directory
    println!("Creating root directory...");
    let perms = Permissions::new(true, true, true);
    let root_dir = disk.create_directory(0, perms).unwrap();
    println!("  Root directory created at block {}\n", root_dir);
    
    // Create some files
    println!("Creating files...");
    let file1 = disk.create_file(1, perms).unwrap();
    let file2 = disk.create_file(2, perms).unwrap();
    println!("  File 1 created at block {}", file1);
    println!("  File 2 created at block {}\n", file2);
    
    // Add files to root directory
    println!("Adding files to root directory...");
    let entry1 = DirectoryEntry::new(1, FileType::File, "readme.txt".to_string()).unwrap();
    let entry2 = DirectoryEntry::new(2, FileType::File, "data.bin".to_string()).unwrap();
    
    disk.add_directory_entry(root_dir, entry1).unwrap();
    disk.add_directory_entry(root_dir, entry2).unwrap();
    println!("  Added 2 files to root\n");
    
    // Create a subdirectory
    println!("Creating subdirectory...");
    let sub_dir = disk.create_directory(3, perms).unwrap();
    let dir_entry = DirectoryEntry::new(3, FileType::Directory, "documents".to_string()).unwrap();
    disk.add_directory_entry(root_dir, dir_entry).unwrap();
    println!("  Subdirectory 'documents' created at block {}\n", sub_dir);
    
    // List root directory contents
    println!("Listing root directory contents:");
    let entries = disk.list_directory(root_dir).unwrap();
    for entry in &entries {
        println!("  - {} (inode={}, type={:?})", entry.name, entry.inode_number, entry.file_type);
    }
    println!();
    
    // Find a specific entry
    println!("Finding 'readme.txt'...");
    let found = disk.find_directory_entry(root_dir, "readme.txt").unwrap();
    println!("  Found: inode={}, type={:?}\n", found.inode_number, found.file_type);
    
    // Add files to subdirectory
    println!("Adding files to subdirectory...");
    let file3 = disk.create_file(4, perms).unwrap();
    let file4 = disk.create_file(5, perms).unwrap();
    
    let entry3 = DirectoryEntry::new(4, FileType::File, "report.pdf".to_string()).unwrap();
    let entry4 = DirectoryEntry::new(5, FileType::File, "notes.txt".to_string()).unwrap();
    
    disk.add_directory_entry(sub_dir, entry3).unwrap();
    disk.add_directory_entry(sub_dir, entry4).unwrap();
    println!("  Added 2 files to 'documents'\n");
    
    // List subdirectory contents
    println!("Listing 'documents' directory:");
    let sub_entries = disk.list_directory(sub_dir).unwrap();
    for entry in &sub_entries {
        println!("  - {} (inode={}, type={:?})", entry.name, entry.inode_number, entry.file_type);
    }
    println!();
    
    // Get directory info
    println!("Getting directory info...");
    let dir_info = disk.get_directory_info(root_dir).unwrap();
    println!("  Root directory:");
    println!("    Inode number: {}", dir_info.inode_number);
    println!("    Type: {:?}", dir_info.file_type);
    println!("    Block count: {}", dir_info.block_count);
    println!("    Entries: {}\n", entries.len());
    
    // Remove an entry
    println!("Removing 'data.bin' from root...");
    let removed_inode = disk.remove_directory_entry(root_dir, "data.bin").unwrap();
    println!("  Removed entry with inode {}\n", removed_inode);
    
    // List root again
    println!("Listing root directory after removal:");
    let entries_after = disk.list_directory(root_dir).unwrap();
    for entry in &entries_after {
        println!("  - {} (inode={}, type={:?})", entry.name, entry.inode_number, entry.file_type);
    }
    println!();
    
    // Display disk statistics
    println!("Disk Statistics:");
    println!("  Used blocks: {}", disk.used_blocks_count());
    println!("  Free blocks: {}", disk.free_blocks_count());
    println!("  Utilization: {:.2}%\n", disk.utilization());
    
    // Try to delete non-empty directory (should fail)
    println!("Attempting to delete non-empty directory...");
    match disk.delete_directory(root_dir) {
        Ok(_) => println!("  ERROR: Should have failed!"),
        Err(e) => println!("  ✓ Correctly rejected: {}\n", e),
    }
    
    // Delete empty subdirectory after removing its contents
    println!("Cleaning up subdirectory...");
    disk.remove_directory_entry(sub_dir, "report.pdf").unwrap();
    disk.remove_directory_entry(sub_dir, "notes.txt").unwrap();
    disk.delete_directory(sub_dir).unwrap();
    println!("  ✓ Subdirectory deleted\n");
    
    // Final statistics
    println!("Final Disk Statistics:");
    println!("  Used blocks: {}", disk.used_blocks_count());
    println!("  Free blocks: {}", disk.free_blocks_count());
    println!("  Utilization: {:.2}%", disk.utilization());
    
    println!("\n✓ Directory operations test completed successfully!");
}
