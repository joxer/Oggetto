extern crate clap;
use clap::{App, Arg};

use oggetto::block::Block;
use oggetto::chunk::Chunk;
use oggetto::redundant_file::{ChunkIndirection, RedundantFile};
use oggetto::volume::{BigFileVolume, Volume};
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
    let mut volume = BigFileVolume::init("volume.bin", "block.bin");
    let id = volume.destruct_from_file("tests/lenna.png").unwrap();
}
