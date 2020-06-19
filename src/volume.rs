use crate::block::Block;
use crate::chunk::Chunk;
use crate::error::VolumeError;
use crate::redundant_file::RedundantFile;
use crate::volume_manager::FileVolumeManager;
use crate::UUID;
use std::collections::HashMap;

pub trait Volume {
    fn get_redundant_file(&self, id: UUID) -> Result<Box<RedundantFile>, VolumeError>;
    fn get_chunk(&self, id: UUID) -> Result<Box<Chunk>, VolumeError>;
    fn get_block(&self, id: UUID) -> Result<Box<Block>, VolumeError>;
    fn destruct_from_file(&mut self, file_name: &str) -> Result<UUID, VolumeError>;
    fn restruct_to_file(&mut self, id: UUID, file_name: &str) -> Result<(), VolumeError>;
}

pub struct BigFileVolume {
    meta_data: Option<FileVolumeManager>,
    block_file: Option<FileVolumeManager>,
}

pub struct BigFileVolumeHashMap<T> {
    hashmap: HashMap<UUID, T>,
}

impl<T> BigFileVolumeHashMap<T> {
    pub fn new() -> BigFileVolumeHashMap<T> {
        BigFileVolumeHashMap {
            hashmap: HashMap::new(),
        }
    }

    pub fn insert(&mut self, id: UUID, value: T) -> Option<T> {
        self.hashmap.insert(id, value)
    }

    pub fn get(&self, id: &UUID) -> Option<&T> {
        self.hashmap.get(&id)
    }
}

impl BigFileVolume {
    pub fn default() -> BigFileVolume {
        return BigFileVolume {
            meta_data: None,
            block_file: None,
        };
    }

    pub fn init(meta_data: &str, block_file: &str) -> BigFileVolume {
        let fvm = match FileVolumeManager::open_metadata(meta_data) {
            Ok(fvm) => fvm,
            Err(_) => FileVolumeManager::init_metadata(meta_data).unwrap(),
        };

        let mut bfv = BigFileVolume::default();

        bfv.meta_data = Some(fvm);

        bfv
    }

    pub fn destruct<T>(&mut self, file: &str, reader: &mut T) -> Result<UUID, VolumeError>
    where
        T: std::io::Read,
    {
        let (file, chunks, blocks) = RedundantFile::destruct(file, reader).unwrap();

        let id = file.id;
        let pos = self.meta_data.as_mut().unwrap().allocate_file(id)?;
        self.meta_data.as_mut().unwrap().sync_metadata();

        self.meta_data
            .as_mut()
            .unwrap()
            .save_file(pos, *file.clone());

        /*self.files.insert(file.id, file);
        for c in chunks.iter() {
            self.chunks.insert(c.id, Box::new(*c));
        }
        for b in blocks.iter() {
            self.blocks.insert(b.id, Box::new(*b));
        }
        */
        //    self.save_file(file);
        //    self.save_chunks(chunks);
        //    self.save_blocks(blocks);

        Ok(file.id)
    }

    /*    fn save_file(&mut self, file: Box<RedundantFile>) {
        self.write_to_file_table(&file);
        self.write_file_block();
    }

    fn save_chunks(&mut self, chunks: Box<Vec<Chunk>>) {
        self.write_chunks_to_file_table(&chunks);
        self.write_chunks_block(&chunks);
    }

    fn save_blocks(&mut self, blocks: Box<Vec<Block>>) {
        self.write_blocks_to_file_table(&blocks);
        self.write_blocks_to_second_file(&blocks)
    }

    fn write_to_file_table(&mut self,file: Box<RedundantFile>) -> Result<(),VolumeError> {

    }
    */

    pub fn restruct<T>(&mut self, id: UUID, writer: &mut T) -> Result<(), VolumeError>
    where
        T: std::io::Write,
    {
        let file = RedundantFile::rebuild(id, self, writer);
        writer.flush();
        Ok(())
    }
}

impl Volume for BigFileVolume {
    fn get_redundant_file(&self, id: UUID) -> Result<Box<RedundantFile>, VolumeError> {
        //    let file = self.files.get(&id).unwrap();
        //    Ok(file.clone())
        Err(VolumeError::GeneralError)
    }
    fn get_chunk(&self, id: UUID) -> Result<Box<Chunk>, VolumeError> {
        //let chunk = self.chunks.get(&id).unwrap();
        //Ok(chunk.clone())
        Err(VolumeError::GeneralError)
    }
    fn get_block(&self, id: UUID) -> Result<Box<Block>, VolumeError> {
        //let block = self.blocks.get(&id).unwrap();
        //Ok(block.clone())
        Err(VolumeError::GeneralError)
    }

    fn destruct_from_file(&mut self, file_name: &str) -> Result<UUID, VolumeError> {
        let mut file = std::fs::File::open(file_name).unwrap();

        self.destruct(file_name, &mut file)
    }

    fn restruct_to_file(&mut self, id: UUID, file_name: &str) -> Result<(), VolumeError> {
        let mut file = std::fs::File::create(file_name).unwrap();

        self.restruct(id, &mut file);
        Ok(())
    }
}
