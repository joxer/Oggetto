use crate::crc32c::crc32c;
use crate::data_encoding::HEXUPPER;
use crate::reed_solomon_erasure::galois_8::ReedSolomon;
use crate::serde::ser::SerializeSeq;
use crate::serde::Serializer;

use crate::constants::{BLOCKS, BLOCK_SIZE, PARITY, READ_STEP};
use crate::error::{RedundantFileError, VolumeError};
use crate::uuid::Uuid;
use crate::volume::Volume;
use crate::UUID;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

use crate::block::Block;

#[derive(Serialize, Deserialize, Copy, Clone)]
pub struct Chunk {
    pub id: u128,
    pub position: u32,
    pub chunk_n: usize,
    pub parity_n: usize,
    pub chunk_size: usize,
    pub blocks: [UUID; BLOCKS + PARITY], //[Block;BLOCKS+PARITY],
    pub hash: u32,
}

/*
fn block_id_serialize<S>(blocks: &Vec<Box<Block>>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq = s.serialize_seq(Some(128))?;
    for b in blocks {
        seq.serialize_element(&b.id.to_le_bytes());
    }
    seq.end()
}*/

impl Default for Chunk {
    fn default() -> Self {
        return Chunk {
            id: 0,
            position: 0,
            chunk_n: BLOCKS,
            parity_n: PARITY,
            chunk_size: READ_STEP,
            blocks: [0u128; BLOCKS + PARITY],
            hash: 0,
        };
    }
}

pub fn chunk_block_serialize(chunk: &Chunk, blocks: &Vec<Block>) -> Vec<u8> {
    let mut ret = Vec::new();
    let mut chunk_serialized = bincode::serialize(chunk).unwrap();
    ret.append(&mut chunk_serialized);
    for block in blocks {
        let mut serialized_block = bincode::serialize(block).unwrap();
        ret.append(&mut serialized_block);
    }
    ret
}

impl Chunk {
    pub fn size() -> usize {
        let chunk_size = bincode::serialized_size(&Chunk::default()).unwrap() as usize;
        let block_size = bincode::serialized_size(&Block::default()).unwrap() as usize;

        chunk_size + block_size * (BLOCKS + PARITY)
    }

    pub fn rebuild<T, W>(id: UUID, data_manager: &T, writer: &mut W) -> Result<(), VolumeError>
    where
        T: Volume,
        W: std::io::Write,
    {
        let chunk: Box<Chunk> = data_manager.get_chunk(id)?;
        chunk.inner_rebuild(data_manager, writer);
        Ok(())
    }

    pub fn inner_rebuild<T, W>(&self, data_manager: &T, writer: &mut W) -> Result<(), VolumeError>
    where
        T: Volume,
        W: std::io::Write,
    {
        let blocks: Vec<Box<Block>> = self
            .blocks
            .iter()
            .map(|b| match data_manager.get_block(*b) {
                Ok(data) => data,
                Err(err) => {
                    println!("{} missing block", *b);
                    Box::new(Block::empty())
                }
            })
            .collect();

        let data = Chunk::rebuild_data(
            self.position,
            self.chunk_size,
            self.chunk_n,
            self.parity_n,
            self.hash,
            blocks,
        )
        .unwrap();

        writer.write(&data[..]);

        Ok(())
    }

    #[allow(clippy::needless_range_loop)]
    fn rebuild_data(
        position: u32,
        chunk_size: usize,
        chunk_n: usize,
        parity_n: usize,
        hash: u32,
        blocks: Vec<Box<Block>>,
    ) -> Result<Vec<u8>, RedundantFileError> {
        let r: ReedSolomon = ReedSolomon::new(chunk_n, parity_n).unwrap();

        let mut chunks_sliced: Vec<_> = blocks[0..(chunk_n + parity_n)]
            .iter()
            .map(|x| x.inner_data_as_vec())
            .collect();

        r.reconstruct(&mut chunks_sliced)
            .map_err(RedundantFileError::RecostructError)?;

        let mut vec = Vec::<u8>::new();
        for c in chunks_sliced {
            vec.extend(c.unwrap());
        }

        let sliced_hash: u32 = crc32c(&vec[0..chunk_size]);

        if sliced_hash != hash {
            let hash_string = format!("{}", sliced_hash);
            let hash_chunk = format!("{}", hash);
            return Err(RedundantFileError::MismatchHash(
                position,
                hash_string,
                hash_chunk,
            ));
        }

        Ok(Vec::from(&vec[0..chunk_size]))
    }

    pub fn empty() -> Chunk {
        Chunk {
            id: 0,
            position: 0,
            chunk_n: 0,
            parity_n: 0,
            chunk_size: 0,
            blocks: [0u128; BLOCKS + PARITY],
            hash: 0,
        }
    }
    pub fn build(
        buf: &[u8; READ_STEP],
        read_bytes: usize,
        position: u32,
        n_chunks: usize,
    ) -> Result<(Box<Chunk>, Box<Vec<Block>>), RedundantFileError> {
        let r = ReedSolomon::new(BLOCKS, PARITY).unwrap();

        let hash_slice: u32 = crc32c(&buf[0..read_bytes]);

        let _length = buf.len();
        let mut vecs: Vec<Vec<u8>> = Vec::new();

        let mut start = 0;
        while start < read_bytes {
            let mut v = Vec::from(&buf[start..std::cmp::min(start + BLOCK_SIZE, read_bytes)]);
            if v.len() < BLOCK_SIZE {
                v.append(&mut vec![0u8; BLOCK_SIZE - v.len()]);
            }
            vecs.push(v);
            start += BLOCK_SIZE;
        }
        for i in 0..(PARITY + (BLOCKS - vecs.len())) {
            let v: Vec<u8> = ([0u8; BLOCK_SIZE]).to_vec();
            vecs.push(v);
        }

        r.encode(&mut vecs).unwrap();
        let blocked_chunks: Vec<Block> = vecs
            .iter()
            .enumerate()
            .map(|(pos, data)| {
                let mut array = [0u8; BLOCK_SIZE];
                for i in 0..data.len() {
                    array[i] = data[i]
                }
                let data_crc = crc32c(&data);
                Block {
                    id: Uuid::new_v4().as_u128(),
                    position: pos,
                    data: array,
                    crc: data_crc,
                }
            })
            .collect();
        let mut blocked_array = [0u128; BLOCKS + PARITY];
        for i in 0..blocked_chunks.len() {
            blocked_array[i] = blocked_chunks[i].id;
        }

        Ok((
            Box::new(Chunk {
                id: Uuid::new_v4().as_u128(),
                position,
                chunk_n: BLOCKS,
                parity_n: PARITY,
                chunk_size: read_bytes,
                blocks: blocked_array,
                hash: hash_slice,
            }),
            Box::new(blocked_chunks),
        ))
    }
}
