use file_system_simulator::virtual_disk::VirtualDisk;

fn main() {
    println!("=== File System Simulator ===\n");
    
    let mut disk = VirtualDisk::new("/Users/anishtimalsina/Desktop/projects/file_system_simulator/test.txt").unwrap();

    // Print disk statistics
    println!("Disk Statistics:");
    println!("  Total blocks: {}", disk.total_blocks());
    println!("  Used blocks: {}", disk.used_blocks_count());
    println!("  Free blocks: {}", disk.free_blocks_count());
    println!("  Utilization: {:.2}%\n", disk.utilization());

    // Allocate some blocks
    let block1 = disk.allocate_block().unwrap();
    println!("Allocated block: {}", block1);

    let block2 = disk.allocate_block().unwrap();
    println!("Allocated block: {}", block2);

    // Allocate contiguous blocks
    let contiguous_start = disk.allocate_contiguous_blocks(5).unwrap();
    println!("Allocated 5 contiguous blocks starting at: {}", contiguous_start);

    // Free some blocks
    disk.free_block(block1).unwrap();
    println!("Freed block: {}", block1);

    disk.free_blocks(contiguous_start, 5).unwrap();
    println!("Freed 5 blocks starting at: {}", contiguous_start);

    // Print disk statistics
    println!("Disk Statistics:");
    println!("  Total blocks: {}", disk.total_blocks());
    println!("  Used blocks: {}", disk.used_blocks_count());
    println!("  Free blocks: {}", disk.free_blocks_count());
    println!("  Utilization: {:.2}%\n", disk.utilization());
}

