extern crate clap;
use clap::{App, Arg};

use oggetto::redundant_file::RedundantFile;
use oggetto::volume::BigFileVolume;
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
    //let mut file = std::fs::File::open("tests/lenna.png").unwrap();
    let mut volume = BigFileVolume::default();
    let id = volume.destruct_from_file("tests/salinger.mp3").unwrap();
    let id = volume.restruct_to_file(id,"tests/salinger_r.mp3").unwrap();

    
    
}
