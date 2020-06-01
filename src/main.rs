extern crate clap;
use clap::{App, Arg, SubCommand};
use oggetto::file;
use std::path::Path;

fn main() {
    let matches = App::new("Oggetto")
        .subcommand(
            SubCommand::with_name("write").arg(
                Arg::with_name("file")
                    .short("f")
                    .required(true)
                    .help("file to write to rocksdb"),
            ),
        )
        .subcommand(
            SubCommand::with_name("read")
                .arg(Arg::with_name("file").short("f").help("file to read")),
        )
        .get_matches();
    if let Some(matches) = matches.subcommand_matches("write") {
        let input = matches.value_of("input").unwrap();
        let path = Path::new(input);
        let file = file::RedudantFile::destruct(path).unwrap();
        let file_r = file
            .rebuild()
            .unwrap()
            .iter()
            .map(|&c| c as char)
            .collect::<String>();
        println!("{}", file_r);
        file.store().unwrap();
    }
    if let Some(matches) = matches.subcommand_matches("read") {
        let input = matches.value_of("input").unwrap();
        let file = file::RedudantFile::restore(input.to_owned()).unwrap();
        let file_r = file
            .rebuild()
            .unwrap()
            .iter()
            .map(|&c| c as char)
            .collect::<String>();
        println!("{}", file_r);
    }
}
