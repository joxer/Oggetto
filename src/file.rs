use crate::rocksdb::{DB};
use serde::ser::{ SerializeStruct, Serializer};
use std::fs::File;
use std::io::{Read};
use std::path::{Path};

use crate::chunk::{Chunk, deserialize, local_chunk::LocalChunk};
use crate::error::{RedundantFileError};
use crate::constants::{READ_STEP, ROCKS_DB_PATH, };

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
        let chunks: Vec<String> = self.chunks.iter().map(|x| x.serialize()).collect();
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
        let tree = path.to_str().unwrap()[0..path.to_str().unwrap().len() - name.len()].to_owned();
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
            position += 1;
        }

        Ok(RedudantFile {
            name: name.to_str().unwrap().to_owned(),
            path: tree,
            chunks,
        })
    }

    fn save_chunks(&self) -> Result<(), RedundantFileError> {
        for c in &self.chunks {
            c.save_chunks()?;
        }
        Ok(())
    }
    pub fn store(&self) -> Result<(), RedundantFileError> {
        let db = DB::open_default(ROCKS_DB_PATH).unwrap();
        
        db.put(&self.name, serde_json::to_string(self).map_err(RedundantFileError::JSONError)?).map_err(RedundantFileError::RocksDBError)?;
        self.save_chunks()?;

        Ok(())
    }

    pub fn restore(path: String) -> Result<RedudantFile, RedundantFileError> {
        let db = DB::open_default(ROCKS_DB_PATH).unwrap();
        let db_value = db.get(path).unwrap().unwrap();
        let value: serde_json::Value =
            serde_json::from_str(&String::from_utf8(db_value).unwrap()).unwrap();
        let mut chunks = Vec::new();

        let j_chunks: Vec<Box<dyn Chunk>> = value
            .get("chunks")
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .map(|x| deserialize(&x.as_str().unwrap().to_owned()))
            .collect();
        for i in j_chunks {
            chunks.push(i);
        }

        Ok(RedudantFile {
            name: value.get("name").unwrap().as_str().unwrap().to_owned(),
            path: value.get("path").unwrap().as_str().unwrap().to_owned(),
            chunks: chunks,
        })
    }

    pub fn rebuild(&self) -> Result<Vec<u8>, RedundantFileError> {
        let mut vec = Vec::<u8>::new();
        for c in &self.chunks {
            vec.extend(c.rebuild()?);
        }

        Ok(vec)
    }
}
