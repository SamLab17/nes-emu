use nes_emu::ines::parse::INesFile;
use std::path::Path;
use std::{fs, env};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let path = &env::args().collect::<Vec<String>>()[1];
    let bytes = fs::read(Path::new(&path))?;
    println!("Reading {path}");
    let res = INesFile::parse_from(&bytes);
    match res {
        Ok((_, file)) => {
            println!("Valid INES File!");
            println!("Header: {:?}", file.header);
        },
        Err(e) => {
            println!("Invalid INES 2 File: {:?}", e);
        }
    }
    Ok(())
}