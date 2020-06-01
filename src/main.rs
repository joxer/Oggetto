extern crate clap;
use clap::{App, Arg};
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
            Some(input) => {
                let path = Path::new(input);

                let ret = oggetto::redundant_file::RedundantFile::destruct(path).unwrap();
                let file_r = ret
                    .rebuild()
                    .unwrap()
                    .iter()
                    .map(|&c| c as char)
                    .collect::<String>();
                println!("{}", file_r);
                ret.store().unwrap();
            }
            None => {
                println!("no file specified");
            }
        }
    }
    if let Some(ref matches) = matches.subcommand_matches("read") {
        match matches.value_of("FILE") {
            Some(input) => {
                match oggetto::redundant_file::RedundantFile::restore(input.to_owned()) {
                    Ok(ret) => {
                        let file_r = ret
                            .rebuild()
                            .unwrap()
                            .iter()
                            .map(|&c| c as char)
                            .collect::<String>();
                        println!("{}", file_r);
                    }
                    Err(error) => {
                        println!("{:?}", error);
                    }
                }
            }
            None => {
                println!("no file specified");
            }
        }
    }
}
