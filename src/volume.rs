use crate::uuid::Uuid;
use error::BlockError;
use redundant_file::RedundantFile;

pub trait Volume {
    
    write(pos: usize, block: &[u8]) -> Result<usize, VolumeError>;
    read(pos: usize) -> Result<Block,VolumeError>;
    get_redundant_files() -> Result<Vec<RedundantFile>,VolumeError>;
}

pub struct BigFileVolume {
    /*
        first 10 megs are for redundant files structure
    */ 
    path: Path,    
}


