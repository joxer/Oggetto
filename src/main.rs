extern crate clap;
use clap::{App, Arg};

use oggetto::volume::{BigFileVolume, Volume};

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
            Some(input) => {
                let mut volume = BigFileVolume::init("volume.bin", "block.bin");
                let _id = volume.destruct_from_file(input).unwrap();
            }
            None => {
                println!("no file specified");
            }
        }
    }
    if let Some(ref matches) = matches.subcommand_matches("read") {
        match matches.value_of("FILE") {
            Some(_input) => {}
            None => {
                println!("no file specified");
            }
        }
    }
}
