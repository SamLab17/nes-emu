use crate::ines::parse::INesFile;

use crate::error::Result;

use super::cart::Cartridge;
use super::nrom::build_nrom_cart;


pub fn build_cartridge(rom: &INesFile) -> Result<Cartridge> {
    match rom.header.mapper {
        0 => build_nrom_cart(&rom.prg_rom, &rom.chr_rom, rom.header.mirror_type),
        _ => Err("ROM uses an unsupported mapper".into())
    }
}