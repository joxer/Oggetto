use crate::constants::CHUNK_SIZE;
use crate::crc32c::crc32c;
use crate::uuid::Uuid;
use serde::{Deserialize, Serialize};

#[repr(align(32))]
#[derive(Debug, Serialize, Deserialize)]
pub struct Block {
    pub id: u128,
    pub position: usize,
    pub data: Box<[u8]>, // CHUNK_SIZE
    pub crc: u32,
}
impl Block {
    pub fn inner_data_as_vec(&self) -> Option<Vec<u8>> {
        let crc = crc32c(&*self.data);
        if crc != self.crc {
            return None;
        }
        return Some(self.data.clone().to_vec());
    }

    pub fn parity(position: usize) -> Box<Block> {
        let mut v: Vec<u8> = Vec::new();
        v.resize(CHUNK_SIZE, 0u8);
        let crc = crc32c(&v);
        Box::new(Block {
            id: Uuid::new_v4().as_u128(),
            data: v.into_boxed_slice(),
            position: position,
            crc: crc,
        })
    }
}
