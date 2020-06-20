use crate::block::Block;
use crate::chunk::Chunk;
use crate::constants::FIRST_INDIRECTION_SIZE;
use crate::constants::{BLOCKS, FILENAME_SIZE, READ_STEP};
use crate::error::RedundantFileError;
use crate::error::VolumeError;
use crate::serde::{Deserialize, Serialize};
use crate::volume::Volume;
use crate::UUID;
use serde::ser::{SerializeSeq, Serializer};
use std::alloc::{alloc, dealloc, Layout};
use std::fs::File;
use std::io::Read;
use std::path::Path;

#[derive(Serialize, Deserialize, Clone)]
pub struct RedundantFile {
    pub id: UUID,
    pub name: Box<[u8]>, // MAX FILENAME SIZE 256
    pub chunks_fi: Box<ChunkIndirection>,
    pub chunks_si: Box<[ChunkIndirection; FIRST_INDIRECTION_SIZE]>, //#[serde(serialize_with = "chunk_id_serialize")]
                                                                    //#[serde(deserialize_with = "chunk_id_derialize")]
                                                                    //pub chunks_fi: Box<[Chunk;FIRST_INDIRECTION_SIZE]>,
                                                                    //#[serde(serialize_with = "chunk_id_serialize_of_first_indirection")]
                                                                    //pub chunks_u16: Box<[Box<[Chunk;16]>;16]>
}

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, PartialOrd)]
pub struct ChunkIndirection {
    pub chunks: [UUID; FIRST_INDIRECTION_SIZE],
}

impl Default for ChunkIndirection {
    fn default() -> ChunkIndirection {
        return ChunkIndirection {
            chunks: [0u128; FIRST_INDIRECTION_SIZE],
        };
    }
}

impl ChunkIndirection {
    fn to_bin_vec(self) -> Vec<u8> {
        let vecs_uuid: Vec<u128> = self.chunks.to_vec();
        let mut vecs_splitted = Vec::new();
        for v in vecs_uuid {
            for i in 0..(128 / 8) {
                vecs_splitted.push((v >> i * 8) as u8 & 0b11111111)
            }
        }

        vecs_splitted
    }
}

pub struct Directory {}
/*
fn chunk_id_serialize_of_first_indirection<S>(
    chunks: &Box<[Box<[Chunk; 16]>; 16]>,
    s: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq = s.serialize_seq(Some(128))?;
    for v_chunks in chunks.iter() {
        for o_chunk in v_chunks.iter() {
            seq.serialize_element(&o_chunk.id.to_le_bytes());
        }
    }
    seq.end()
}

fn chunk_id_serialize<S>(chunks: & Box<[Chunk; FIRST_INDIRECTION_SIZE]>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq = s.serialize_seq(Some(FIRST_INDIRECTION_SIZE))?;
    for o_chunk in chunks.iter() {
        seq.serialize_element(&o_chunk.id.to_le_bytes());
    }
    seq.end()
}

fn chunk_id_deserialize<'de, D>(data: D) -> Result<[Chunk; FIRST_INDIRECTION_SIZE], D::Error>
where
    D: Deserializer<'de>,
{

    let array: &[u8] = Deserialize::deserialize(data)?;
    let mut buf = [0u8;BLOCK_SIZE];
    for i in 0..array.len() {
        buf[i] = array[i];
    }

    Ok(buf)
}
*/

impl RedundantFile {
    pub fn size() -> usize {
        let rf = std::mem::size_of::<RedundantFile>();
        let fi = std::mem::size_of::<ChunkIndirection>();
        let si = std::mem::size_of::<ChunkIndirection>() * FIRST_INDIRECTION_SIZE;
        rf + fi + si + (FILENAME_SIZE as usize)
    }

    pub fn rebuild<T, W>(id: UUID, data_manager: &T, writer: &mut W) -> Result<(), VolumeError>
    where
        T: Volume,
        W: std::io::Write,
    {
        let file: Box<RedundantFile> = data_manager.get_redundant_file(id)?;
        file.inner_rebuild(data_manager, writer)?;
        Ok(())
    }

    pub fn inner_rebuild<T, W>(&self, data_manager: &T, writer: &mut W) -> Result<(), VolumeError>
    where
        T: Volume,
        W: std::io::Write,
    {
        for c in self.chunks_fi.chunks.iter() {
            if *c != 0 {
                Chunk::rebuild(*c, data_manager, writer)?;
            }
        }

        for cs in self.chunks_si.iter() {
            for c in cs.chunks.iter() {
                if *c != 0 {
                    Chunk::rebuild(*c, data_manager, writer)?;
                }
            }
        }

        Ok(())
    }

    pub fn destruct<T>(
        file: &str,
        reader: &mut T,
    ) -> Result<(Box<RedundantFile>, Box<Vec<Chunk>>, Box<Vec<Block>>), RedundantFileError>
    where
        T: std::io::Read,
    {
        let mut chunks = Vec::<Chunk>::new();
        let mut blocks = Vec::<Block>::new();
        let mut position = 0;
        loop {
            let mut buf = Box::new([0; READ_STEP]);
            let n = reader.read(&mut buf[..]).map_err(RedundantFileError::Io)?;

            let (chunk, mut c_blocks) = Chunk::build(&buf, n, position, BLOCKS).unwrap();
            chunks.push(*chunk);
            blocks.append(&mut c_blocks);
            if n < READ_STEP {
                break;
            }
            position += 1;
        }
        let mut chunks_fi: [u128; FIRST_INDIRECTION_SIZE] = [0; FIRST_INDIRECTION_SIZE];
        for n in 0..std::cmp::min(FIRST_INDIRECTION_SIZE, chunks.len()) {
            chunks_fi[n] = chunks[n].id;
        }

        let mut name_u8 = Box::new([0u8; FILENAME_SIZE]);
        for (n, x) in file.chars().enumerate() {
            name_u8[n] = x as u8;
        }

        let mut chunks_si: [ChunkIndirection; FIRST_INDIRECTION_SIZE] =
            [ChunkIndirection::default(); FIRST_INDIRECTION_SIZE];
        if chunks.len() > FIRST_INDIRECTION_SIZE {
            let missing = chunks.len() - FIRST_INDIRECTION_SIZE;

            let mut i = 0;
            let mut k = 0;
            for j in 0..missing {
                chunks_si[i].chunks[k] = chunks[FIRST_INDIRECTION_SIZE + j].id;

                k += 1;
                if k == FIRST_INDIRECTION_SIZE {
                    i += 1;
                    k = 0;
                }
            }
        }
        Ok((
            Box::new(RedundantFile {
                id: uuid::Uuid::new_v4().as_u128(),
                name: name_u8,
                chunks_fi: Box::new(ChunkIndirection { chunks: chunks_fi }),
                chunks_si: Box::new(chunks_si),
            }),
            Box::new(chunks),
            Box::new(blocks),
        ))
    }
}
/*
impl Into<Vec<u8>> for RedundantFile {
    fn into(self) -> Vec<u8> {
        let mut buf: Vec<u8> = Vec::new();
        let values: Vec<Vec<u8>> = vec![
            self.id.to_le_bytes().to_vec(),
            self.name.to_vec(),
            self.chunks_fi.to_bin_vec(),
            //   self.chunk_vector_start.to_le_bytes().to_vec()];
            self.chunks_si.iter().flat_map(|x| x.to_bin_vec()).collect(),
        ];



        buf
    }
}
*/
