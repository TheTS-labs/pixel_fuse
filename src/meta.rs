use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct MetaFrame {
    pub filename: String,
    pub remainder: usize,
    pub hash: String,
    pub version: String,
    pub frames_count: usize,
}

impl MetaFrame {
    pub fn into_bytes(&self) -> Vec<u8> {
        bincode::serialize(&self).unwrap()
    }

    pub fn from_bytes(bytes: &Vec<u8>) -> MetaFrame {
        bincode::deserialize(bytes).unwrap()
    }
}