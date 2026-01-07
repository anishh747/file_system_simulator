use file_system_simulator::cli::FileSystemShell;

fn main() {
    let mut shell = FileSystemShell::new("/Users/anishtimalsina/Desktop/projects/file_system_simulator/test.txt")
        .expect("Failed to initialize file system");
    
    if let Err(e) = shell.run() {
        eprintln!("Shell error: {}", e);
        std::process::exit(1);
    }
}
