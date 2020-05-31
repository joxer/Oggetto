extern crate clap;
use clap::{Arg, App};
use oggetto::file;
use std::path::Path;

fn main() {

    let matches = App::new("Oggetto")
                          .arg(Arg::with_name("INPUT")
                               .help("Sets the input file to use")
                               .required(true)
                               .index(1))
                          .get_matches();
    let input = matches.value_of("INPUT").unwrap();
    
    let path = Path::new(input);
    let file = file::RedudantFile::destruct(path).unwrap();
    let file_r = file.rebuild().unwrap().iter().map(|&c| c as char).collect::<String>();
    println!("{}",file_r);
    file.store().unwrap();
    //let file = file::RedudantFile::restore(path.to_str().unwrap().to_owned()).unwrap();
    //let file_r = file.rebuild().unwrap().iter().map(|&c| c as char).collect::<String>();
    //println!("{}",file_r);
    
    //println!("{:#?}", file.chunks);
}
