use crate::error::RedundantFileError;
use std::fmt::Debug;
pub mod local_chunk;

pub trait Chunk: Debug {
    fn position(&self) -> u32;
    fn chunk_size(&self) -> usize;
    fn rebuild(&self) -> Result<Vec<u8>, RedundantFileError>;

    fn serialize(&self) -> String;
    fn save_chunks(&self) -> Result<(), RedundantFileError>;
}

pub fn deserialize(obj: &str) -> Box<dyn Chunk> {
    let json: serde_json::Value = serde_json::from_str(obj).unwrap();
    match json.get("type").unwrap().as_str().unwrap() {
        "LocalChunk" => local_chunk::LocalChunk::deserialize_json(json),
        _ => panic!("Cannot handle file"),
    }
}
