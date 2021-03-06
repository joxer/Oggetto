extern crate bincode;
extern crate crc32c;
extern crate data_encoding;

extern crate reed_solomon_erasure;

extern crate serde;
extern crate serde_derive;
extern crate serde_json;

extern crate uuid;
pub mod block;
pub mod chunk;
pub mod constants;
pub mod error;
pub mod redundant_file;
pub mod volume;
pub mod volume_manager;

type UUID = u128;
