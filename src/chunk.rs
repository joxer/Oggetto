use crate::crc32c::crc32c;
use crate::data_encoding::HEXUPPER;
use crate::reed_solomon_erasure::galois_8::ReedSolomon;
use crate::serde::ser::SerializeSeq;
use crate::serde::Serializer;
use crate::sha2::{Digest, Sha256};
use crate::uuid::Uuid;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

use crate::constants::{CHUNKS, PARITY, READ_STEP};
use crate::error::RedundantFileError;

use crate::block::Block;

#[repr(align(32))]
#[derive(Debug, Serialize, Deserialize)]
pub struct Chunk {
    pub id: u128,
    pub position: u32,
    pub chunk_n: usize,
    pub parity_n: usize,
    pub chunk_size: usize,
    #[serde(serialize_with = "block_id_serialize")]
    pub chunks: Vec<Box<Block>>,
    pub hash: [u8; 32],
}

fn block_id_serialize<S>(blocks: &Vec<Box<Block>>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq = s.serialize_seq(Some(128))?;
    for b in blocks {
        seq.serialize_element(&b.id.to_le_bytes());
    }
    seq.end()
}
/*
impl Chunk  {
    fn position(&self) -> u32 {
        self.position
    }

    fn chunk_size(&self) -> usize {
        self.chunk_size
    }
    #[allow(clippy::needless_range_loop)]
    fn rebuild(&self) -> Result<Vec<u8>, RedundantFileError> {
        let r: ReedSolomon = ReedSolomon::new(self.chunk_n, self.parity_n).unwrap();

        let mut chunks_sliced: Vec<_> = self.chunks[0..(self.chunk_n + self.parity_n)]
            .iter()
            .map(|x| x.inner_data())
            .collect();

        r.reconstruct(&mut chunks_sliced)
            .map_err(RedundantFileError::RecostructError)?;

        let mut vec = Vec::<u8>::new();
        for c in chunks_sliced {
            vec.extend(c.unwrap());
        }

        let sliced_hash: [u8; 32] = Sha256::digest(&vec[0..self.chunk_size()]).into();

        let hash_string = HEXUPPER.encode(&sliced_hash);
        if hash_string != self.hash {
            return Err(RedundantFileError::MismatchHash(
                self.position(),
                hash_string,
                self.hash.clone(),
            ));
        }

        Ok(Vec::from(&vec[0..self.chunk_size]))
    }

    fn serialize(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}*/

impl Chunk {
    pub fn build(
        buf: &[u8; READ_STEP],
        read_bytes: usize,
        position: u32,
        n_chunks: usize,
    ) -> Result<Box<Chunk>, RedundantFileError> {
        let r = ReedSolomon::new(CHUNKS, PARITY).unwrap();

        let hash_slice: [u8; 32] = Sha256::digest(&buf[0..read_bytes]).into();

        let _length = buf.len();
        let mut vecs: Vec<Vec<u8>> = (0..n_chunks)
            .map(|x| {
                Vec::from(&buf[(read_bytes / n_chunks * x)..(read_bytes / n_chunks * (x + 1))])
            })
            .collect();
        r.encode(&mut vecs).unwrap();
        let blocked_chunks = vecs
            .iter()
            .enumerate()
            .map(|(pos, data)| {
                let data_crc = crc32c(&data);
                Box::new(Block {
                    id: Uuid::new_v4(),
                    position: pos,
                    data: data.clone().into_boxed_slice(),
                    crc: data_crc,
                })
            })
            .collect();
        Ok(Box::new(Chunk {
            uuid: Uuid::new_v4(),
            position,
            chunk_n: CHUNKS,
            parity_n: PARITY,
            chunk_size: read_bytes,
            chunks: blocked_chunks,
            hash: hash_slice,
        }))
    }
}
