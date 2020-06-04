use crate::rocksdb::DB;
use serde::ser::{SerializeSeq, Serializer};
use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::chunk::Chunk;
use crate::constants::{CHUNKS, READ_STEP};
use crate::error::RedundantFileError;
use crate::serde::{Deserialize, Serialize};

#[repr(align(16))]
#[derive(Debug, Serialize, Deserialize)]
pub struct RedundantFile {
    pub name: Box<[u8]>, // MAX FILENAME SIZE 256
    #[serde(serialize_with = "chunk_id_serialize")]
    pub chunks_u8: [Option<Box<Chunk>>;8],
    #[serde(serialize_with = "chunk_id_serialize_of_first_indirection")]
    pub chunks_u16: [[Option<Box<Chunk>>;16];16],
}

pub struct Directory {

}

fn chunk_id_serialize_of_first_indirection<S>(chunks: & [[Option<Box<Chunk>>;16];16], s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq = s.serialize_seq(Some(128))?;
    for v_chunks in chunks {
        for o_chunk in v_chunks {
            match o_chunk {
                Some(chunk) => {
            seq.serialize_element(&chunk.id.to_le_bytes());
                },
                None => {
                    seq.serialize_element(&0u128);
                }
            }

        }
    }
    seq.end()
    
}

fn chunk_id_serialize<S>(chunks: & [Option<Box<Chunk>>; 8], s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq = s.serialize_seq(Some(128))?;
    for o_chunk in chunks {
        match o_chunk {
         Some(chunk)    => {
        seq.serialize_element(&chunk.id.to_le_bytes());
         },
         None => {
             seq.serialize_element(&0u128);
         }
        }
    }
    seq.end()
}

impl RedundantFile {
    pub fn destruct<T>(
        file: &str,
        path: &str,
        reader: &mut T,
    ) -> Result<RedundantFile, RedundantFileError>
    where
        T: std::io::Read,
    {
        let mut chunks = Vec::<Box<Chunk>>::new();
        let mut position = 0;
        loop {
            let mut buf = Box::new([0; READ_STEP]);
            let n = reader.read(&mut buf[..]).map_err(RedundantFileError::Io)?;

            let chunk = Chunk::build(&buf, n, position, CHUNKS).unwrap();
            chunks.push(chunk);
            if n < READ_STEP {
                break;
            }
            position += 1;
        }
        let mut chunks_8: [Option<Box<Chunk>>; 8] = [None;8];
        for n in 0..chunks.len(){
            chunks_8[n] = Some(chunks.remove(n));
        }

        let mut name_u8 = [0u8;256];
        for (n,x) in file.chars().enumerate() {
            name_u8[n] = x as u8;
        }

        Ok(RedundantFile {
            name: Box::from(name_u8),
            chunks_u8: chunks_8,
            chunks_u16: [ [None;16];16] //for now let's put null
        })
    }

}
