use crate::error::VolumeError;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::ops::Deref;
use crate::redundant_file::RedundantFile;
use crate::UUID;
pub struct FileVolumeManager {
    path: String,
    file: Option<File>,
    super_block: SuperBlock,
    file_vector: Vec<FileVector>,
    
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SuperBlock {
    file_size: u64,
    file_vector_start: u64,
    
}

impl Default for SuperBlock {
    fn default() -> SuperBlock {
        SuperBlock {
            file_size: u64::pow(8, 9),
            file_vector_start: std::mem::size_of::<SuperBlock>() as u64,
        }
    }
}

impl Into<Vec<u8>> for SuperBlock {
    fn into(self) -> Vec<u8> {
        let mut buf: Vec<u8> = Vec::new();
        let i = 0;
        let values: Vec<Vec<u8>> = vec![
            self.file_size.to_le_bytes().to_vec(),
            self.file_vector_start.to_le_bytes().to_vec(),
            //   self.chunk_vector_start.to_le_bytes().to_vec()];
        ];
        for v in values {
            for b in v.iter() {
                buf.push(*b);
            }
        }

        buf
    }
}

impl From<[u8; std::mem::size_of::<SuperBlock>()]> for SuperBlock {
    fn from(bytes: [u8; std::mem::size_of::<SuperBlock>()]) -> Self {
        let mut buf = [0u8; 8];
        buf.clone_from_slice(&bytes[0..8]);
        let size: u64 = u64::from_le_bytes(buf);

        buf.clone_from_slice(&bytes[8..16]);
        let start: u64 = u64::from_le_bytes(buf);

        SuperBlock {
            file_size: size,
            file_vector_start: start,
        }
    }
}

#[derive(Copy, Clone)]
pub struct FileVector {
    entries: [(u64, u128); 4096],
    next_file_vector: u64,
}

impl Default for FileVector {
    fn default() -> Self {
        FileVector {
            entries: [(0u64, 0u128); 4096],
            next_file_vector: 0,
        }
    }
}

impl Into<Vec<u8>> for FileVector {
    fn into(self) -> Vec<u8> {
        let mut buf: Vec<u8> = Vec::new();
        let i = 0;

        for e in self.entries.iter() {
            for b in e.0.to_le_bytes().to_vec() {
                buf.push(b);
            }
            for b in e.1.to_le_bytes().to_vec() {
                buf.push(b);
            }
        }
        for b in self.next_file_vector.to_le_bytes().to_vec() {
            buf.push(b);
        }

        buf
    }
}

impl From<[u8; std::mem::size_of::<FileVector>()]> for FileVector {
    fn from(bytes: [u8; std::mem::size_of::<FileVector>()]) -> Self {
        let entries = [(0u64, 0u128); 4096];
        let mut i = 0;
        while i < (8 + 16) * 4096 {
            let mut buf = [0u8; 8];
            buf.clone_from_slice(&bytes[i..i + 8]);
            let size: u64 = u64::from_le_bytes(buf);

            let mut buf = [0u8; 16];
            buf.clone_from_slice(&bytes[i + 8..i + 8 + 16]);
            let size: u128 = u128::from_le_bytes(buf);

            i += 8 + 16;
        }

        let mut buf = [0u8; 8];
        buf.clone_from_slice(&bytes[(8 + 16) * 4096..(8 + 16) * 4096 + 8]);
        let next_file_vector = u64::from_le_bytes(buf);
        FileVector {
            entries: entries,
            next_file_vector: next_file_vector,
        }
    }
}

impl FileVolumeManager {
    pub fn init(path: &str) -> Result<FileVolumeManager, VolumeError> {
        let mut file = File::create(path).map_err(VolumeError::IoError)?;
        let mut bytes_representation: Vec<u8> = SuperBlock::default().into();
        file.write(&bytes_representation[..]);
        bytes_representation = FileVector::default().into();
        file.write(&bytes_representation[..]);

        FileVolumeManager::open(path)
    }

    pub fn open(path: &str) -> Result<FileVolumeManager, VolumeError> {
        let mut file = File::open(path).map_err(VolumeError::IoError)?;
        let mut buf_sb = [0u8; std::mem::size_of::<SuperBlock>()];
        file.read_exact(&mut buf_sb).map_err(VolumeError::IoError)?;
        let sb: SuperBlock = buf_sb.into();
        let mut buf_fv = [0u8; std::mem::size_of::<FileVector>()];
        file.read_exact(&mut buf_fv).map_err(VolumeError::IoError)?;
        let fv: FileVector = buf_fv.into();
        let mut v_fv = Vec::new();
        v_fv.push(fv);
        Ok(FileVolumeManager {
            path: path.to_owned(),
            file: Some(file),
            super_block: sb,
            file_vector: v_fv,
        })
    }

    pub fn sync_metadata(&mut self) -> Result<(), VolumeError> {
        let mut seek = self.super_block.file_vector_start;
        for b_fv in &self.file_vector {
            let mut buf_fv = [0u8; std::mem::size_of::<FileVector>()];
            let fv_v: Vec<u8> = b_fv.clone().into();
            buf_fv.clone_from_slice(&fv_v[..]);
            self.file
                .as_mut()
                .unwrap()
                .seek(SeekFrom::Start(seek))
                .map_err(VolumeError::IoError)?;
            seek = b_fv.next_file_vector;
        }
        Ok(())
    }

    pub fn allocate_file(&mut self, id: UUID) -> Result<u64,VolumeError> {
        let mut pos = 0;
        let mut pos_start=self.super_block.file_vector_start;
        for file_vector in self.file_vector.iter_mut() {
            for (n,data) in file_vector.entries.iter_mut().enumerate() {
            if *data == (0u64,0u128) {
                pos = pos_start + (n*RedundantFile::size()) as u64;
                data.0 = pos;
                data.1 = id;
                break;
            }
            
        }
            pos_start = file_vector.next_file_vector;
        }
        Ok(pos)
    }

    pub fn save_file(&mut self, pos: u64, rf: Box<RedundantFile>) -> Result<(),VolumeError> {

        self.file.as_mut().unwrap().seek(SeekFrom::Start(pos));
        let buf = Vec::<u8>::new();
        

        Ok(())
    }

}
