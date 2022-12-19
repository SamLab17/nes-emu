use nes_emu::ines::parse::INesFile;
use std::path::Path;
use std::process::exit;
use std::{fs, env};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let path = &env::args().nth(1).expect("No path provided.");
    let bytes = fs::read(Path::new(&path))?;
    println!("Reading {path}");
    let file = INesFile::try_from(&bytes)?;
    println!("Valid INES File!");
    println!("Header: {:?}", file.header);
    exit(0);
}