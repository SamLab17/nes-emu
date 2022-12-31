use crate::ines::parse::INesFile;

use crate::error::Result;

use super::cart::Cartridge;
use super::mapper0::build_nrom_cart;
use super::mapper1::build_mmc1_cart;


pub fn build_cartridge(rom: &INesFile) -> Result<Cartridge> {
    match rom.header.mapper {
        0 => build_nrom_cart(&rom.prg_rom, &rom.chr_rom, rom.header.mirror_type),
        1 => build_mmc1_cart(&rom.prg_rom, &rom.chr_rom),
        _ => Err("ROM uses an unsupported mapper".into())
    }
}