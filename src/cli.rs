use crate::error::{FsError, FsResult};
use crate::serialization::{DirectoryEntry, FileType, Permissions};
use crate::virtual_disk::VirtualDisk;
use colored::*;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::collections::HashMap;

/// File System Shell - Interactive CLI for the file system simulator
pub struct FileSystemShell {
    disk: VirtualDisk,
    current_dir: u64,           // Current directory inode block
    current_path: String,       // Current path for display
    inode_map: HashMap<String, u64>, // Path -> inode block mapping
}

impl FileSystemShell {
    pub fn new(disk_path: &str) -> FsResult<Self> {
        let mut disk = VirtualDisk::new(disk_path)?;
        
        // Initialize root directory using create_directory which properly allocates entries block
        let perms = Permissions::new(true, true, true);
        let root_block = disk.create_directory(0, perms)?;
        
        let mut inode_map = HashMap::new();
        inode_map.insert("/".to_string(), root_block);
        
        Ok(FileSystemShell {
            disk,
            current_dir: root_block,
            current_path: "/".to_string(),
            inode_map,
        })
    }

    pub fn run(&mut self) -> FsResult<()> {
        println!("{}", "=== File System Simulator - Interactive Shell ===".bright_cyan().bold());
        println!("{}", "Type 'help' for available commands, 'exit' to quit\n".bright_black());
        
        let mut rl = DefaultEditor::new().map_err(|e| {
            FsError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
        })?;
        
        loop {
            let prompt = format!("{}{}{}> ", 
                "fs".bright_green().bold(), 
                ":".bright_black(),
                self.current_path.bright_blue());
            
            match rl.readline(&prompt) {
                Ok(line) => {
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }
                    
                    rl.add_history_entry(line).ok();
                    
                    if line == "exit" || line == "quit" {
                        println!("{}", "Goodbye!".bright_green());
                        break;
                    }
                    
                    if let Err(e) = self.execute_command(line) {
                        println!("{} {}", "Error:".bright_red().bold(), e);
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    println!("{}", "^C".bright_yellow());
                    continue;
                }
                Err(ReadlineError::Eof) => {
                    println!("{}", "exit".bright_yellow());
                    break;
                }
                Err(err) => {
                    println!("{} {:?}", "Error:".bright_red().bold(), err);
                    break;
                }
            }
        }
        
        Ok(())
    }

    fn execute_command(&mut self, line: &str) -> FsResult<()> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(());
        }
        
        let cmd = parts[0];
        let args = &parts[1..];
        
        match cmd {
            "help" => self.cmd_help(),
            "ls" => self.cmd_ls(args),
            "pwd" => self.cmd_pwd(),
            "mkdir" => self.cmd_mkdir(args),
            "touch" => self.cmd_touch(args),
            "cat" => self.cmd_cat(args),
            "echo" => self.cmd_echo(args),
            "rm" => self.cmd_rm(args),
            "rmdir" => self.cmd_rmdir(args),
            "stat" => self.cmd_stat(args),
            "df" => self.cmd_df(),
            "tree" => self.cmd_tree(),
            "clear" => self.cmd_clear(),
            _ => {
                println!("{} Unknown command: '{}'. Type 'help' for available commands.", 
                         "Error:".bright_red().bold(), cmd);
                Ok(())
            }
        }
    }

    fn cmd_help(&self) -> FsResult<()> {
        println!("\n{}", "Available Commands:".bright_cyan().bold());
        println!("  {}  - List directory contents", "ls".bright_green());
        println!("  {}  - Print working directory", "pwd".bright_green());
        println!("  {}  - Create directory", "mkdir <name>".bright_green());
        println!("  {}  - Create empty file", "touch <name>".bright_green());
        println!("  {}  - Display file contents", "cat <file>".bright_green());
        println!("  {}  - Write text to file", "echo <text> > <file>".bright_green());
        println!("  {}  - Remove file", "rm <file>".bright_green());
        println!("  {}  - Remove empty directory", "rmdir <dir>".bright_green());
        println!("  {}  - Show file/directory info", "stat <name>".bright_green());
        println!("  {}  - Show disk usage", "df".bright_green());
        println!("  {}  - Show directory tree", "tree".bright_green());
        println!("  {}  - Clear screen", "clear".bright_green());
        println!("  {}  - Exit shell", "exit".bright_green());
        println!();
        Ok(())
    }

    fn cmd_ls(&mut self, _args: &[&str]) -> FsResult<()> {
        let entries = self.disk.list_directory(self.current_dir)?;
        
        if entries.is_empty() {
            println!("{}", "  (empty)".bright_black());
            return Ok(());
        }
        
        for entry in entries {
            let icon = match entry.file_type {
                FileType::Directory => "📁",
                FileType::File => "📄",
            };
            
            let name = match entry.file_type {
                FileType::Directory => entry.name.bright_blue().bold(),
                FileType::File => entry.name.normal(),
            };
            
            println!("  {} {}", icon, name);
        }
        
        Ok(())
    }

    fn cmd_pwd(&self) -> FsResult<()> {
        println!("{}", self.current_path.bright_blue());
        Ok(())
    }

    fn cmd_mkdir(&mut self, args: &[&str]) -> FsResult<()> {
        if args.is_empty() {
            return Err(FsError::InvalidPath("Usage: mkdir <name>".to_string()));
        }
        
        let name = args[0];
        
        // Check if already exists
        if self.disk.find_directory_entry(self.current_dir, name).is_ok() {
            return Err(FsError::AlreadyExists(name.to_string()));
        }
        
        // Create directory
        let inode_num = self.get_next_inode_number();
        let perms = Permissions::new(true, true, true);
        let dir_block = self.disk.create_directory(inode_num, perms)?;
        
        // Add to current directory
        let entry = DirectoryEntry::new(inode_num, FileType::Directory, name.to_string())?;
        self.disk.add_directory_entry(self.current_dir, entry)?;
        
        // Update inode map
        let full_path = self.join_path(name);
        self.inode_map.insert(full_path, dir_block);
        
        println!("{} Created directory '{}'", "✓".bright_green(), name.bright_blue());
        Ok(())
    }

    fn cmd_touch(&mut self, args: &[&str]) -> FsResult<()> {
        if args.is_empty() {
            return Err(FsError::InvalidPath("Usage: touch <name>".to_string()));
        }
        
        let name = args[0];
        
        // Check if already exists
        if self.disk.find_directory_entry(self.current_dir, name).is_ok() {
            return Err(FsError::AlreadyExists(name.to_string()));
        }
        
        // Create file
        let inode_num = self.get_next_inode_number();
        let perms = Permissions::new(true, true, false);
        let file_block = self.disk.create_file(inode_num, perms)?;
        
        // Add to current directory
        let entry = DirectoryEntry::new(inode_num, FileType::File, name.to_string())?;
        self.disk.add_directory_entry(self.current_dir, entry)?;
        
        // Update inode map
        let full_path = self.join_path(name);
        self.inode_map.insert(full_path, file_block);
        
        println!("{} Created file '{}'", "✓".bright_green(), name);
        Ok(())
    }

    fn cmd_cat(&mut self, args: &[&str]) -> FsResult<()> {
        if args.is_empty() {
            return Err(FsError::InvalidPath("Usage: cat <file>".to_string()));
        }
        
        let name = args[0];
        let entry = self.disk.find_directory_entry(self.current_dir, name)?;
        
        if entry.file_type != FileType::File {
            return Err(FsError::NotAFile(name.to_string()));
        }
        
        let full_path = self.join_path(name);
        let file_block = self.inode_map.get(&full_path)
            .ok_or_else(|| FsError::FileNotFound(name.to_string()))?;
        
        let data = self.disk.read_file(*file_block)?;
        let content = String::from_utf8_lossy(&data);
        println!("{}", content);
        
        Ok(())
    }

    fn cmd_echo(&mut self, args: &[&str]) -> FsResult<()> {
        if args.len() < 3 || args[args.len() - 2] != ">" {
            return Err(FsError::InvalidPath("Usage: echo <text> > <file>".to_string()));
        }
        
        let filename = args[args.len() - 1];
        let text: Vec<&str> = args[..args.len() - 2].to_vec();
        let content = text.join(" ");
        
        // Find or create file
        let (file_block, is_new) = match self.disk.find_directory_entry(self.current_dir, filename) {
            Ok(entry) => {
                if entry.file_type != FileType::File {
                    return Err(FsError::NotAFile(filename.to_string()));
                }
                let full_path = self.join_path(filename);
                let block = *self.inode_map.get(&full_path)
                    .ok_or_else(|| FsError::FileNotFound(filename.to_string()))?;
                (block, false)
            }
            Err(_) => {
                // Create new file
                let inode_num = self.get_next_inode_number();
                let perms = Permissions::new(true, true, false);
                let file_block = self.disk.create_file(inode_num, perms)?;
                
                let entry = DirectoryEntry::new(inode_num, FileType::File, filename.to_string())?;
                self.disk.add_directory_entry(self.current_dir, entry)?;
                
                let full_path = self.join_path(filename);
                self.inode_map.insert(full_path, file_block);
                
                (file_block, true)
            }
        };
        
        // Write content
        self.disk.write_file(file_block, content.as_bytes())?;
        
        if is_new {
            println!("{} Created and wrote to '{}'", "✓".bright_green(), filename);
        } else {
            println!("{} Wrote to '{}'", "✓".bright_green(), filename);
        }
        
        Ok(())
    }

    fn cmd_rm(&mut self, args: &[&str]) -> FsResult<()> {
        if args.is_empty() {
            return Err(FsError::InvalidPath("Usage: rm <file>".to_string()));
        }
        
        let name = args[0];
        let entry = self.disk.find_directory_entry(self.current_dir, name)?;
        
        if entry.file_type != FileType::File {
            return Err(FsError::NotAFile(format!("'{}' is not a file", name)));
        }
        
        let full_path = self.join_path(name);
        let file_block = *self.inode_map.get(&full_path)
            .ok_or_else(|| FsError::FileNotFound(name.to_string()))?;
        
        // Delete file
        self.disk.delete_file(file_block)?;
        
        // Remove from directory
        self.disk.remove_directory_entry(self.current_dir, name)?;
        
        // Remove from inode map
        self.inode_map.remove(&full_path);
        
        println!("{} Removed file '{}'", "✓".bright_green(), name);
        Ok(())
    }

    fn cmd_rmdir(&mut self, args: &[&str]) -> FsResult<()> {
        if args.is_empty() {
            return Err(FsError::InvalidPath("Usage: rmdir <dir>".to_string()));
        }
        
        let name = args[0];
        let entry = self.disk.find_directory_entry(self.current_dir, name)?;
        
        if entry.file_type != FileType::Directory {
            return Err(FsError::NotADirectory(format!("'{}' is not a directory", name)));
        }
        
        let full_path = self.join_path(name);
        let dir_block = *self.inode_map.get(&full_path)
            .ok_or_else(|| FsError::DirectoryNotFound(name.to_string()))?;
        
        // Delete directory (must be empty)
        self.disk.delete_directory(dir_block)?;
        
        // Remove from parent directory
        self.disk.remove_directory_entry(self.current_dir, name)?;
        
        // Remove from inode map
        self.inode_map.remove(&full_path);
        
        println!("{} Removed directory '{}'", "✓".bright_green(), name);
        Ok(())
    }

    fn cmd_stat(&mut self, args: &[&str]) -> FsResult<()> {
        if args.is_empty() {
            return Err(FsError::InvalidPath("Usage: stat <name>".to_string()));
        }
        
        let name = args[0];
        let _entry = self.disk.find_directory_entry(self.current_dir, name)?;
        
        let full_path = self.join_path(name);
        let inode_block = *self.inode_map.get(&full_path)
            .ok_or_else(|| FsError::FileNotFound(name.to_string()))?;
        
        let inode = self.disk.read_inode(inode_block)?;
        
        println!("\n{} {}", "File:".bright_cyan().bold(), name.bright_white());
        println!("  {}: {:?}", "Type".bright_cyan(), inode.file_type);
        println!("  {}: {}", "Inode".bright_cyan(), inode.inode_number);
        println!("  {}: {} bytes", "Size".bright_cyan(), inode.size);
        println!("  {}: {}", "Blocks".bright_cyan(), inode.block_count);
        println!("  {}: {}", "Links".bright_cyan(), inode.link_count);
        println!();
        
        Ok(())
    }

    fn cmd_df(&self) -> FsResult<()> {
        let total = self.disk.total_blocks();
        let used = self.disk.used_blocks_count();
        let free = self.disk.free_blocks_count();
        let utilization = self.disk.utilization();
        
        println!("\n{}", "Disk Usage:".bright_cyan().bold());
        println!("  {}: {}", "Total blocks".bright_cyan(), total);
        println!("  {}: {} ({:.2}%)", "Used blocks".bright_cyan(), used, utilization);
        println!("  {}: {}", "Free blocks".bright_cyan(), free);
        println!("  {}: 4 KB", "Block size".bright_cyan());
        println!("  {}: {} MB", "Total size".bright_cyan(), (total * 4) / 1024);
        println!();
        
        Ok(())
    }

    fn cmd_tree(&mut self) -> FsResult<()> {
        println!("\n{}", self.current_path.bright_blue().bold());
        self.print_tree(self.current_dir, "", true)?;
        println!();
        Ok(())
    }

    fn print_tree(&mut self, dir_block: u64, prefix: &str, _is_last: bool) -> FsResult<()> {
        let entries = self.disk.list_directory(dir_block)?;
        let count = entries.len();
        
        for (i, entry) in entries.iter().enumerate() {
            let is_last_entry = i == count - 1;
            let connector = if is_last_entry { "└── " } else { "├── " };
            let new_prefix = if is_last_entry { "    " } else { "│   " };
            
            let icon = match entry.file_type {
                FileType::Directory => "📁",
                FileType::File => "📄",
            };
            
            let name = match entry.file_type {
                FileType::Directory => entry.name.bright_blue(),
                FileType::File => entry.name.normal(),
            };
            
            println!("{}{}{} {}", prefix, connector, icon, name);
            
            // Recursively print subdirectories
            if entry.file_type == FileType::Directory {
                let full_path = format!("{}/{}", self.current_path.trim_end_matches('/'), entry.name);
                if let Some(&sub_dir_block) = self.inode_map.get(&full_path) {
                    let new_full_prefix = format!("{}{}", prefix, new_prefix);
                    self.print_tree(sub_dir_block, &new_full_prefix, is_last_entry)?;
                }
            }
        }
        
        Ok(())
    }

    fn cmd_clear(&self) -> FsResult<()> {
        print!("\x1B[2J\x1B[1;1H");
        Ok(())
    }

    fn join_path(&self, name: &str) -> String {
        if self.current_path == "/" {
            format!("/{}", name)
        } else {
            format!("{}/{}", self.current_path, name)
        }
    }

    fn get_next_inode_number(&self) -> u64 {
        self.inode_map.len() as u64
    }
}
