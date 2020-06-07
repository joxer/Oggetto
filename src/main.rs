extern crate clap;
use clap::{App, Arg};

use oggetto::block::Block;
use oggetto::chunk::Chunk;
use oggetto::redundant_file::{ChunkIndirection,RedundantFile};
use oggetto::volume::{BigFileVolume,Volume};
use oggetto::volume_manager::FileVolumeManager;
use std::path::Path;
fn main() {
    let matches = App::new("Oggetto")
        .subcommand(
            App::new("write").arg(
                Arg::with_name("FILE")
                    .index(1)
                    .required(true)
                    .help("file to write to rocksdb"),
            ),
        )
        .subcommand(
            App::new("read").arg(
                Arg::with_name("FILE")
                    .index(1)
                    .required(true)
                    .help("file to read"),
            ),
        )
        .get_matches();
    if let Some(ref matches) = matches.subcommand_matches("write") {
        match matches.value_of("FILE") {
            Some(input) => {}
            None => {
                println!("no file specified");
            }
        }
    }
    if let Some(ref matches) = matches.subcommand_matches("read") {
        match matches.value_of("FILE") {
            Some(input) => {}
            None => {
                println!("no file specified");
            }
        }
    }
    let mut file = std::fs::File::open("tests/lenna.png").unwrap();
     let mut volume = BigFileVolume::default();
    let id = volume.destruct_from_file("tests/lenna.png").unwrap();
    //let id = volume.restruct_to_file(id,"tests/salinger_r.mp3").unwrap();
    
    println!("{}", std::mem::size_of::<RedundantFile>());
    println!("{}", RedundantFile::size());
     println!("{}", std::mem::size_of::<Chunk>());
    println!("{}", std::mem::size_of::<Block>());
    let ci = ChunkIndirection { 
        chunks: [0u128; 32]
    };
    let r = RedundantFile {
        id: 0u128,
        name: ([99u8;256].to_vec().into_boxed_slice()),
        chunks_fi: Box::new(ci),
        chunks_si: Box::new([ci;32 ],
        
    
    )};
    println!("{:#?}", bincode::serialized_size(&r).unwrap());
    
    //FileVolumeManager::init("volume.bin");
    
}
