use crate::metadata::FileMetadata;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Directory {
    pub metadata: FileMetadata,
    pub contents: HashMap<String, FileMetadata>,
}

impl Directory {
    pub fn new(metadata: FileMetadata) -> Directory {
        Directory {
            metadata,
            contents: HashMap::new(),
        }
    }

    pub fn add_entry(&mut self, metadata: FileMetadata) {
        let name = metadata.name.clone();
        self.contents.insert(name, metadata);
    }

    pub fn remove_entry(&mut self, name: &str) {
        self.contents.remove(name);
    }

    pub fn list_entries(&self) {
        for (name, metadata) in &self.contents {
            println!("{} {:?}", name, metadata)
        }
    }
}
