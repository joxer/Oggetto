use crate::reed_solomon_erasure::galois_8::ReedSolomon;
use crate::sha2::{Sha256, Digest};
use crate::data_encoding::HEXUPPER;
use crate::crc32c::crc32c;
use crate::rocksdb::{DB, Options};
use crate::uuid::Uuid;
use serde::ser::{Serialize, SerializeStruct, Serializer};
use serde::de::{self, Deserialize, Deserializer, Visitor, SeqAccess, MapAccess};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path,PathBuf};
use std::fmt::Debug;
const PARITY: usize = 128;
const CHUNKS: usize = 4;
const READ_STEP: usize = 4*4096*CHUNKS;
const ROCKS_DB_PATH: &str = "tmp/rocksdb";
const DIRECTORIES: [&str; 2] = ["path1", "path2" ];
pub trait Chunk: Debug {
    fn position(&self) -> u32;
    fn chunk_size(&self) -> usize;
    fn rebuild(&self) -> Result<Vec<u8>,RedundantFileError>;

    fn serialize(&self) -> String;
    fn save_chunks(&self) -> Result<(), RedundantFileError>;
    
}
fn deserialize<'a>(obj: &'a String) -> Box<dyn Chunk> {
    let json: serde_json::Value = serde_json::from_str(obj).unwrap();
    match json.get("type").unwrap().as_str().unwrap() {
        "LocalChunk" => {
            LocalChunk::deserialize_json(json)
        }
        _ => {
            panic!("Cannot handle file")
        }
        
    }

}

#[derive(Debug)]
pub struct LocalChunk {
    
    uuid: Uuid,
    position: u32,
    chunk_n: usize,
    parity_n: usize,
    chunk_size: usize,
    chunks: Vec<Vec<u8>>,
    chunks_crc: Vec<u32>,
    hash: String,
}

impl serde::Serialize for LocalChunk {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        
        let mut chunk = serializer.serialize_struct("LocalChunk", 10)?;
        chunk.serialize_field("type", &"LocalChunk")?;
        chunk.serialize_field("uuid", &self.uuid)?;
        chunk.serialize_field("position", &self.position)?;
        chunk.serialize_field("chunk_n", &self.chunk_n)?;
        chunk.serialize_field("parity_n", &self.parity_n)?;
        chunk.serialize_field("chunk_size", &self.chunk_size)?;
        chunk.serialize_field("chunks", &self.generate_names())?;
        chunk.serialize_field("chunks_crc", &self.chunks_crc)?;
        chunk.serialize_field("hash", &self.hash)?;
        chunk.end()
    }
}

impl Chunk for LocalChunk {
    fn position(&self) -> u32 {
        self.position
    }

    fn chunk_size(&self) -> usize {
        self.chunk_size
    }

    fn rebuild(&self) -> Result<Vec<u8>,RedundantFileError> {
        let r: ReedSolomon = ReedSolomon::new(self.chunk_n, self.parity_n).unwrap();
        
        let mut chunks_sliced: Vec<_> = self.chunks[0..(self.chunk_n+self.parity_n)].iter().map(|x| Some(x.to_owned())).collect();
        
        for c in 0..self.chunk_n {
            if crc32c(&chunks_sliced[c].as_ref().unwrap()) != self.chunks_crc[c] {
                chunks_sliced[c] = None;
                println!("{} is none", c);
            }
        }    
        r.reconstruct(&mut chunks_sliced).map_err(RedundantFileError::RecostructError)?; 
        
        let mut vec =  Vec::<u8>::new();
        for c in chunks_sliced {
            vec.extend(c.unwrap());
        }

        let sliced_hash: [u8; 32] = Sha256::digest(&vec[0..self.chunk_size()]).into();
        
        let hash_string = HEXUPPER.encode(&sliced_hash);
        if hash_string != self.hash {
            return Err(RedundantFileError::MismatchHash(self.position(), hash_string, self.hash.clone()))
        }

        Ok(Vec::from(&vec[0..self.chunk_size]))
    }

    fn serialize(&self) -> String{
        serde_json::to_string(&self).unwrap()
    }

    fn save_chunks(&self) -> Result<(), RedundantFileError> {
        self.save_to_disk();
        Ok(())
    }


}

impl LocalChunk {
    fn deserialize_json(values: serde_json::Value) -> Box<dyn Chunk>{
        let mut t = LocalChunk {
            uuid: Uuid::parse_str(values.get("uuid").unwrap().as_str().unwrap()).unwrap(),
            position: values.get("position").unwrap().as_u64().unwrap() as u32,
            chunk_n: values.get("chunk_n").unwrap().as_u64().unwrap() as usize,
            parity_n:  values.get("parity_n").unwrap().as_u64().unwrap() as usize,
            chunk_size:  values.get("chunk_size").unwrap().as_u64().unwrap() as usize,
            chunks: LocalChunk::read_from_disk(values.get("chunks").unwrap().as_array().unwrap().iter().map(|x| x.as_str().unwrap().to_owned()).collect()),
            hash:  values.get("hash").unwrap().as_str().unwrap().to_owned(),
            chunks_crc: values.get("chunks_crc").unwrap().as_array().unwrap().iter().map(|x| x.as_u64().unwrap() as u32).collect(),
        };
        Box::from(t)
    }

    pub fn build(buf: &[u8;READ_STEP], read_bytes: usize, position: u32) -> Result<LocalChunk, RedundantFileError>{
        let r = ReedSolomon::new(CHUNKS, PARITY).unwrap();

        
        let sliced_hash: [u8; 32] = Sha256::digest(&buf[0..read_bytes]).into();
        
        let hash_string = HEXUPPER.encode(&sliced_hash);
        
        let _length = buf.len();
        let mut vecs : Vec<Vec<u8>> = (0..CHUNKS).map(|x| Vec::from(&buf[(READ_STEP/CHUNKS*x)..(READ_STEP/CHUNKS*(x+1))])).collect();
        let mut crcs =  Vec::<u32>::new();
        for v in &vecs {
            crcs.push(crc32c(&v));
        }
        for _p in 0..PARITY {
            vecs.push([0u8;READ_STEP/CHUNKS].to_vec());
        }
        r.encode(&mut vecs).unwrap();
        
        
        Ok(LocalChunk{
            uuid: Uuid::new_v4(),
            position,
            chunk_n: CHUNKS,
            parity_n: PARITY,
            chunk_size: read_bytes,
            chunks: vecs,
            hash: hash_string,
            chunks_crc: crcs
        })
    }

    fn generate_names(&self) -> Vec<String>{
        let mut v = Vec::new();
        
        for (c_pos, d ) in self.chunks.iter().enumerate() {
            let name = format!("{}/{}-{}",DIRECTORIES[c_pos%DIRECTORIES.len()], self.uuid, c_pos);
            v.push(name);
        }
        
        v
    }

    fn save_to_disk(&self) -> Vec<String> {
        let mut v = Vec::new();
        

            for (c_pos, d ) in self.chunks.iter().enumerate() {
                let name = format!("{}/{}-{}",DIRECTORIES[c_pos%DIRECTORIES.len()], self.uuid, c_pos);
                let mut file = File::create(&name).unwrap();
                file.write_all(d).unwrap();
        
                v.push(name);
            }
            

        v
    }


    fn read_from_disk(paths: Vec<String>) -> Vec<Vec<u8>> {
        let mut v = Vec::new();
        

            for p in paths {
                let path = PathBuf::from(p);
                let mut f = File::open(path).unwrap();
                let mut buffer: Vec<u8> = Vec::new();
                f.read_to_end(&mut buffer).unwrap();
                v.push(buffer);
            }

        v
    }
}

#[derive(Debug)]
pub enum RedundantFileError {
    Io(std::io::Error),
    FileName(std::option::NoneError),
    MismatchHash(u32,String,String),
    RecostructError(reed_solomon_erasure::Error)
}

pub struct RedudantFile {
    pub name: String,
    pub path: String,
    pub chunks: Vec<Box<dyn Chunk>>,
    
}


impl serde::Serialize for RedudantFile {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        
        let mut chunk = serializer.serialize_struct("LocalChunk", 10)?;
        chunk.serialize_field("name", &self.name)?;
        chunk.serialize_field("path", &self.path)?;
        let chunks : Vec<String> = self.chunks.iter().map(|x| x.serialize()).collect();
        chunk.serialize_field("chunks", &chunks)?;
        chunk.end()
    }
}

impl RedudantFile {
    pub fn destruct(path: &Path) -> Result<RedudantFile, RedundantFileError> {
        let mut f = match File::open(path) {
            Err(_) => panic!("couldn't open {}", path.display()),
            Ok(file) => file,
        };
        let name = path.file_name().unwrap();
        let tree = path.to_str().unwrap()[0..path.to_str().unwrap().len()-name.len()].to_owned();
        let mut chunks = Vec::<Box<dyn Chunk>>::new();
        let mut position = 0;
        loop {
            let mut buf = [0; READ_STEP];
            let n = f.read(&mut buf[..]).map_err(RedundantFileError::Io)?;
            

            let chunk = LocalChunk::build(&buf, n, position).unwrap();
            chunks.push(Box::new(chunk));
            if n < READ_STEP {
                break;
            }
            position +=1;
        }

        Ok(RedudantFile {
            name: name.to_str().unwrap().to_owned(),
            path: tree,
            chunks,
        })
    }
    

    fn save_chunks(&self) {
        for c in &self.chunks {
            c.save_chunks();
        }
    }
    pub fn store(&self) -> Result<(),RedundantFileError>{
        let db = DB::open_default(ROCKS_DB_PATH).unwrap();
        //let mut full_path = self.path.clone();
        //full_path.push_str("/");
        

        db.put( &self.name,
            serde_json::to_string(self).unwrap()
        );
        self.save_chunks();
        
        Ok(())
    }

    pub fn restore(path: String) -> Result<RedudantFile,RedundantFileError>{
        let db = DB::open_default(ROCKS_DB_PATH).unwrap();
        let db_value = db.get( path).unwrap().unwrap();
        let value : serde_json::Value = serde_json::from_str(&String::from_utf8(db_value).unwrap()).unwrap();
        let mut chunks = Vec::new();

        let j_chunks: Vec<Box<Chunk>> = value.get("chunks").unwrap().as_array().unwrap().iter().map(|x| deserialize(&x.as_str().unwrap().to_owned())).collect();
        for i in j_chunks {
            chunks.push(i);
        }
        
        Ok(
            RedudantFile{
                name: value.get("name").unwrap().as_str().unwrap().to_owned(),
                path: value.get("path").unwrap().as_str().unwrap().to_owned(),
                chunks: chunks,
            }
        )
    }

    pub fn rebuild(&self) -> Result<Vec<u8>, RedundantFileError> {
        let mut vec =  Vec::<u8>::new();
        for c in &self.chunks {
            vec.extend(c.rebuild()?);
        }
       
        Ok(vec)
    }
}
