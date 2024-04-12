use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockMetadata {
    // pub block_number: u64/* , */
    pub current_position: u64,
}

impl BlockMetadata {
    pub fn new(current_position: u64) -> BlockMetadata {
        BlockMetadata {
            // block_number,
            current_position,
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        serde_json::to_vec(&self).unwrap()
    }
}
