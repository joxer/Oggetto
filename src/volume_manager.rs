use crate::block::Block;
use crate::chunk::{chunk_block_serialize, Chunk};
use crate::error::VolumeError;
use crate::redundant_file::RedundantFile;
use crate::UUID;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};

const file_vector_size: usize = 16;

#[derive(Debug)]
pub enum FileVolumeManager {
    MetaData {
        path: String,
        file: Option<File>,
        super_block: SuperBlock,
        file_vector: Vec<FileVector>,
    },

    BlockFile {
        path: String,
        file: Option<File>,
        super_block: SuperBlock,
        chunk_vector: Vec<ChunkVector>,
    },
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

        let values: Vec<Vec<u8>> = vec![
            self.file_size.to_le_bytes().to_vec(),
            self.file_vector_start.to_le_bytes().to_vec(),
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
    entries: [(u64, u128); file_vector_size],
    next_file_vector: u64,
}

#[derive(Copy, Clone)]
pub struct ChunkVector {
    entries: [(u64, u128); file_vector_size],
    next_chunk_vector: u64,
}

impl std::fmt::Debug for FileVector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FileVector")
            .field(
                "files",
                &self
                    .entries
                    .iter()
                    .map(|t| format!("({} {})", t.0, t.1))
                    .collect::<Vec<String>>()
                    .join(", "),
            )
            .field("next_file_vector", &self.next_file_vector)
            .finish()
    }
}

impl std::fmt::Debug for ChunkVector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FileVector")
            .field(
                "files",
                &self
                    .entries
                    .iter()
                    .map(|t| format!("({} {})", t.0, t.1))
                    .collect::<Vec<String>>()
                    .join(", "),
            )
            .field("next_chunk_vector", &self.next_chunk_vector)
            .finish()
    }
}

impl Default for FileVector {
    fn default() -> Self {
        FileVector {
            entries: [(0u64, 0u128); file_vector_size],
            next_file_vector: 0,
        }
    }
}

impl Default for ChunkVector {
    fn default() -> Self {
        ChunkVector {
            entries: [(0u64, 0u128); file_vector_size],
            next_chunk_vector: 0,
        }
    }
}

impl Into<Vec<u8>> for FileVector {
    fn into(self) -> Vec<u8> {
        let mut buf: Vec<u8> = Vec::new();

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
        let mut entries = [(0u64, 0u128); file_vector_size];
        let mut k = 0;
        let mut i = 0;
        while i < (8 + 16) * file_vector_size {
            let mut buf = [0u8; 8];
            buf.clone_from_slice(&bytes[i..i + 8]);
            let pos: u64 = u64::from_le_bytes(buf);

            let mut buf = [0u8; 16];
            buf.clone_from_slice(&bytes[i + 8..i + 8 + 16]);
            let uuid: u128 = u128::from_le_bytes(buf);

            i += 8 + 16;
            entries[k] = (pos, uuid);
            k += 1;
        }

        let mut buf = [0u8; 8];
        buf.clone_from_slice(&bytes[(8 + 16) * file_vector_size..(8 + 16) * file_vector_size + 8]);
        let next_file_vector = u64::from_le_bytes(buf);
        FileVector {
            entries: entries,
            next_file_vector: next_file_vector,
        }
    }
}

impl Into<Vec<u8>> for ChunkVector {
    fn into(self) -> Vec<u8> {
        let mut buf: Vec<u8> = Vec::new();

        for e in self.entries.iter() {
            for b in e.0.to_le_bytes().to_vec() {
                buf.push(b);
            }
            for b in e.1.to_le_bytes().to_vec() {
                buf.push(b);
            }
        }
        for b in self.next_chunk_vector.to_le_bytes().to_vec() {
            buf.push(b);
        }

        buf
    }
}

impl From<[u8; std::mem::size_of::<ChunkVector>()]> for ChunkVector {
    fn from(bytes: [u8; std::mem::size_of::<FileVector>()]) -> Self {
        let mut entries = [(0u64, 0u128); file_vector_size];
        let mut k = 0;
        let mut i = 0;
        while i < (8 + 16) * file_vector_size {
            let mut buf = [0u8; 8];
            buf.clone_from_slice(&bytes[i..i + 8]);
            let pos: u64 = u64::from_le_bytes(buf);

            let mut buf = [0u8; 16];
            buf.clone_from_slice(&bytes[i + 8..i + 8 + 16]);
            let uuid: u128 = u128::from_le_bytes(buf);

            i += 8 + 16;
            entries[k] = (pos, uuid);
            k += 1;
        }

        let mut buf = [0u8; 8];
        buf.clone_from_slice(&bytes[(8 + 16) * file_vector_size..(8 + 16) * file_vector_size + 8]);
        let next_chunk_vector = u64::from_le_bytes(buf);
        ChunkVector {
            entries: entries,
            next_chunk_vector: next_chunk_vector,
        }
    }
}

impl FileVolumeManager {
    pub fn init_metadata(path: &str) -> Result<FileVolumeManager, VolumeError> {
        let mut file = File::create(path).map_err(VolumeError::IoError)?;
        file.set_len(u64::pow(2, 30))
            .map_err(VolumeError::IoError)?;
        let mut bytes_representation: Vec<u8> = SuperBlock::default().into();
        file.write(&bytes_representation[..])
            .map_err(VolumeError::IoError)?;
        bytes_representation = FileVector::default().into();
        file.write(&bytes_representation[..])
            .map_err(VolumeError::IoError)?;

        FileVolumeManager::open_metadata(path)
    }

    pub fn open_metadata(path: &str) -> Result<FileVolumeManager, VolumeError> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)
            .map_err(VolumeError::IoError)?;
        let mut buf_sb = [0u8; std::mem::size_of::<SuperBlock>()];
        file.read_exact(&mut buf_sb).map_err(VolumeError::IoError)?;
        let sb: SuperBlock = buf_sb.into();
        let mut buf_fv = [0u8; std::mem::size_of::<FileVector>()];

        file.read_exact(&mut buf_fv).map_err(VolumeError::IoError)?;
        let fv: FileVector = buf_fv.into();
        let mut v_fv = Vec::new();
        v_fv.push(fv);
        Ok(FileVolumeManager::MetaData {
            path: path.to_owned(),
            file: Some(file),
            super_block: sb,
            file_vector: v_fv,
        })
    }

    pub fn init_blockdata(path: &str) -> Result<FileVolumeManager, VolumeError> {
        let mut file = File::create(path).map_err(VolumeError::IoError)?;
        file.set_len(u64::pow(2, 30))
            .map_err(VolumeError::IoError)?;
        let mut bytes_representation: Vec<u8> = SuperBlock::default().into();
        file.write(&bytes_representation[..])
            .map_err(VolumeError::IoError)?;
        bytes_representation = ChunkVector::default().into();
        file.write(&bytes_representation[..])
            .map_err(VolumeError::IoError)?;

        FileVolumeManager::open_metadata(path)
    }

    pub fn open_blockdata(path: &str) -> Result<FileVolumeManager, VolumeError> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)
            .map_err(VolumeError::IoError)?;
        let mut buf_sb = [0u8; std::mem::size_of::<SuperBlock>()];
        file.read_exact(&mut buf_sb).map_err(VolumeError::IoError)?;
        let sb: SuperBlock = buf_sb.into();
        let mut buf_fv = [0u8; std::mem::size_of::<FileVector>()];

        file.read_exact(&mut buf_fv).map_err(VolumeError::IoError)?;
        let fv: FileVector = buf_fv.into();
        let mut v_fv = Vec::new();
        v_fv.push(fv);
        Ok(FileVolumeManager::MetaData {
            path: path.to_owned(),
            file: Some(file),
            super_block: sb,
            file_vector: v_fv,
        })
    }

    pub fn sync_metadata(&mut self) -> Result<(), VolumeError> {
        match self {
            FileVolumeManager::MetaData {
                path,
                file,
                super_block,
                file_vector,
            } => {
                let mut seek = super_block.file_vector_start;
                for b_fv in file_vector {
                    let mut buf_fv = [0u8; std::mem::size_of::<FileVector>()];
                    let fv_v: Vec<u8> = b_fv.clone().into();
                    buf_fv.clone_from_slice(&fv_v[..]);
                    file.as_mut()
                        .unwrap()
                        .seek(SeekFrom::Start(seek))
                        .map_err(VolumeError::IoError)?;
                    file.as_mut()
                        .unwrap()
                        .write(&buf_fv[..])
                        .map_err(VolumeError::IoError)?;
                    file.as_mut()
                        .unwrap()
                        .flush()
                        .map_err(VolumeError::IoError)?;
                    seek = b_fv.next_file_vector;
                }
            }
            FileVolumeManager::BlockFile {
                path,
                file,
                super_block,
                chunk_vector,
            } => {
                let mut seek = super_block.file_vector_start;
                for b_fv in chunk_vector {
                    let mut buf_fv = [0u8; std::mem::size_of::<FileVector>()];
                    let fv_v: Vec<u8> = b_fv.clone().into();
                    buf_fv.clone_from_slice(&fv_v[..]);
                    file.as_mut()
                        .unwrap()
                        .seek(SeekFrom::Start(seek))
                        .map_err(VolumeError::IoError)?;
                    file.as_mut()
                        .unwrap()
                        .write(&buf_fv[..])
                        .map_err(VolumeError::IoError)?;
                    file.as_mut()
                        .unwrap()
                        .flush()
                        .map_err(VolumeError::IoError)?;
                    seek = b_fv.next_chunk_vector;
                }
            }
        }
        Ok(())
    }

    pub fn allocate_file(&mut self, id: UUID) -> Result<u64, VolumeError> {
        match self {
            FileVolumeManager::MetaData {
                path,
                file,
                super_block,
                file_vector,
            } => {
                let mut pos = 0;
                let mut pos_start =
                    super_block.file_vector_start + (std::mem::size_of::<FileVector>() as u64);

                for file_vector in file_vector.iter_mut() {
                    for (n, data) in file_vector.entries.iter_mut().enumerate() {
                        if *data == (0u64, 0u128) {
                            pos = pos_start + ((n * RedundantFile::size()) as u64);
                            data.0 = pos;
                            data.1 = id;

                            break;
                        }
                    }
                    pos_start = file_vector.next_file_vector;
                }
                return Ok(pos);
            }
            FileVolumeManager::BlockFile {
                path,
                file,
                super_block,
                chunk_vector,
            } => {
                let mut pos = 0;
                let mut pos_start =
                    super_block.file_vector_start + (std::mem::size_of::<FileVector>() as u64);

                for chunk_vector in chunk_vector.iter_mut() {
                    for (n, data) in chunk_vector.entries.iter_mut().enumerate() {
                        if *data == (0u64, 0u128) {
                            pos = pos_start + ((n * Chunk::size()) as u64);
                            data.0 = pos;
                            data.1 = id;

                            break;
                        }
                    }
                    pos_start = chunk_vector.next_chunk_vector;
                }
                return Ok(pos);
            }
        }
        Ok(0u64)
    }

    pub fn save_file(&mut self, pos: u64, rf: RedundantFile) -> Result<(), VolumeError> {
        match self {
            FileVolumeManager::MetaData {
                path,
                file,
                super_block,
                file_vector,
            } => {
                let fv_v: Vec<u8> = bincode::serialize(&rf).unwrap();
                file.as_mut()
                    .unwrap()
                    .seek(SeekFrom::Start(pos))
                    .map_err(VolumeError::IoError)?;
                file.as_mut()
                    .unwrap()
                    .write(&fv_v[..])
                    .map_err(VolumeError::IoError)?;
                file.as_mut()
                    .unwrap()
                    .flush()
                    .map_err(VolumeError::IoError)?;
            }
            FileVolumeManager::BlockFile {
                path,
                file,
                super_block,
                chunk_vector,
            } => {}
        }
        Ok(())
    }

    pub fn save_chunk(
        &mut self,
        pos: u64,
        chunk: Chunk,
        blocks: Vec<Block>,
    ) -> Result<(), VolumeError> {
        match self {
            FileVolumeManager::MetaData {
                path,
                file,
                super_block,
                file_vector,
            } => {
                let fv_v: Vec<u8> = chunk_block_serialize(&chunk, &blocks);
                file.as_mut()
                    .unwrap()
                    .seek(SeekFrom::Start(pos))
                    .map_err(VolumeError::IoError)?;
                file.as_mut()
                    .unwrap()
                    .write(&fv_v[..])
                    .map_err(VolumeError::IoError)?;
                file.as_mut()
                    .unwrap()
                    .flush()
                    .map_err(VolumeError::IoError)?;
            }
            FileVolumeManager::BlockFile {
                path,
                file,
                super_block,
                chunk_vector,
            } => {}
        }
        Ok(())
    }
}
