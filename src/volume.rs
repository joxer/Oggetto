use crate::block::Block;
use crate::chunk::Chunk;
use crate::error::{RedundantFileError, VolumeError};
use crate::redundant_file::RedundantFile;

use crate::uuid::Uuid;
use serde::ser::{SerializeSeq, Serializer};
use std::fs::File;
use std::io::{Read};
use std::path::Path;
use crate::UUID;
use crate::constants::FIRST_INDIRECTION_SIZE;
use crate::constants::{BLOCKS, READ_STEP};
use crate::serde::{Deserialize, Serialize};
use rand::Rng;
use std::collections::HashMap;

pub trait Volume {
    fn get_redundant_file(&self, id: UUID) -> Result<Box<RedundantFile>, VolumeError>;
    fn get_chunk(&self, id: UUID) -> Result<Box<Chunk>,VolumeError>;
    fn get_block(&self, id: UUID) -> Result<Box<Block>,VolumeError>;
    fn destruct_from_file(&mut self, file_name: &str) -> Result<UUID, VolumeError>;
    fn restruct_to_file(&mut self,id: UUID, file_name: &str) -> Result<(), VolumeError>;
}

pub struct BigFileVolume {
    /*
        first 10 megs are for redundant files structure
    */
    path: String,
    pub files: HashMap<UUID,Box<RedundantFile>>,
    pub chunks: HashMap<UUID,Box<Chunk>>,
    pub blocks: HashMap<UUID,Box<Block>>,

}

impl BigFileVolume {

    pub fn default() -> Box<dyn Volume>{
        return Box::new(
            BigFileVolume{
                path: ".".to_owned(),
                files: HashMap::new(),
                chunks: HashMap::new(),
                blocks: HashMap::new(),
            }
        )
    }


    pub fn destruct<T>(&mut self, file: &str, reader: &mut T) -> Result<UUID , VolumeError> where T: std::io::Read{
        let (file,chunks,blocks) = RedundantFile::destruct(file, reader).unwrap();
        let id = file.id;
        self.files.insert(file.id, file);
        for c in chunks.iter() {
            self.chunks.insert(c.id,Box::new(*c));
        }
        for b in blocks.iter() {
            self.blocks.insert(b.id,Box::new(*b));
        }
        save_file(file);
        save_chunks(chunks);
        save_blocks(blocks);
        
        Ok(id)
    }

    pub fn restruct<T>(&mut self, id: UUID, writer: &mut T) -> Result<() , VolumeError> where T: std::io::Write{
        let file = RedundantFile::rebuild(id,  self, writer);
        writer.flush();
        Ok(())
    }
}

impl Volume for BigFileVolume {

    fn get_redundant_file(&self, id: UUID) -> Result<Box<RedundantFile>, VolumeError> {
        let file = self.files.get(&id).unwrap();
        Ok(file.clone())
    }
    fn get_chunk(&self, id: UUID) -> Result<Box<Chunk>,VolumeError> {
        let chunk = self.chunks.get(&id).unwrap();
        Ok(chunk.clone())
        

    }
    fn get_block(&self, id: UUID) -> Result<Box<Block>,VolumeError>{
        
        let block = self.blocks.get(&id).unwrap();
        Ok(block.clone())
        

    }
   


    fn destruct_from_file(&mut self, file_name: &str) -> Result<UUID, VolumeError>{

        let mut file = std::fs::File::open(file_name).unwrap();

        self.destruct(file_name, &mut file)
    }


    fn restruct_to_file(&mut self, id: UUID, file_name: &str) -> Result<(), VolumeError>{

        let mut file = std::fs::File::create(file_name).unwrap();

        self.restruct(id, &mut file);
        Ok(())
    }

}
