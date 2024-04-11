use file_system_simulator::virtual_disk::VirtualDisk;
fn main() {
    let mut disk = VirtualDisk::new("/home/sxynix/files/test9.txt").unwrap();
    disk.initialize_root_dir().unwrap();
    disk.read_blocks(0).unwrap();
}
