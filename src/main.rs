mod mem;
mod error;
mod cpu;
mod ines;
mod ppu;
mod cart;

use ines::parse::INesFile;

use std::{env, fs, path::Path, error::Error};
use cpu::cpu::Cpu;

use crate::cart::builder::build_cartridge;

fn main() -> Result<(), Box<dyn Error>> {
    let rom_path = env::args().nth(1).expect("No ROM file provided");
    let rom = fs::read(Path::new(&rom_path))?;

    let ines_rom = INesFile::try_from(&rom).expect("Path provided is not a valid NES ROM.");

    let _cpu = Cpu::new(build_cartridge(&ines_rom).expect("This ROM is not supported."));

    Ok(())
}
