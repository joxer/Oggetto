use crate::constants::BLOCK_SIZE;
use crate::crc32c::crc32c;
use crate::uuid::Uuid;
use serde::de::Deserializer;
use serde::ser::{SerializeSeq, Serializer};
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Formatter, Result as fmtResult};
#[derive(Serialize, Deserialize, Copy, Clone)]
pub struct Block {
    pub id: u128,
    pub position: usize,

    #[serde(serialize_with = "data_serialize")]
    #[serde(deserialize_with = "data_deserialize")]
    pub data: [u8; BLOCK_SIZE], // BLOCK_SIZE //generic array https://docs.rs/generic-array/0.14.1/generic_array/
    pub crc: u32,
}

impl Debug for Block {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmtResult {
        f.debug_struct("Block")
            .field("id", &self.id)
            .field("position", &self.position)
            .finish()
    }
}

fn data_serialize<S>(chunks: &[u8; BLOCK_SIZE], s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq = s.serialize_seq(Some(BLOCK_SIZE))?;
    for o_chunk in chunks.iter() {
        seq.serialize_element(o_chunk);
    }
    seq.end()
}

fn data_deserialize<'de, D>(data: D) -> Result<[u8; BLOCK_SIZE], D::Error>
where
    D: Deserializer<'de>,
{
    let array: &[u8] = Deserialize::deserialize(data)?;
    let mut buf = [0u8; BLOCK_SIZE];
    for i in 0..array.len() {
        buf[i] = array[i];
    }

    Ok(buf)
}

impl Block {
    pub fn empty() -> Block {
        Block {
            id: 0,
            position: 0,
            data: [0u8; BLOCK_SIZE],
            crc: 0,
        }
    }

    pub fn inner_data_as_vec(&self) -> Option<Vec<u8>> {
        if self.id == 0 {
            return None;
        }

        let crc = crc32c(&self.data);
        if crc != self.crc {
            return None;
        }
        return Some(self.data.clone().to_vec());
    }

    pub fn parity(position: usize) -> Box<Block> {
        let mut v = [0u8; BLOCK_SIZE];

        let crc = crc32c(&v);
        Box::new(Block {
            id: Uuid::new_v4().as_u128(),
            data: v,
            position: position,
            crc: crc,
        })
    }
}
