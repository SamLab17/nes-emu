use super::cart::Cartridge;

use crate::error::Result;
use crate::mem::device::{MemoryDevice, MemoryError};

#[derive(Debug)]
pub struct Nrom128 {
    prg_rom: [u8; 16 * 1024],
    chr_rom: [u8; 8 * 1024],
}

impl MemoryDevice for Nrom128 {
    fn name(&self) -> String { "NROM-128".into() }

    fn read(&self, addr: u16) -> crate::error::Result<u8> {
        match addr {
            0x8000..=0xFFFF => Ok(self.prg_rom[(addr & 0x3FFF) as usize]),
            _ => Err(Box::new(MemoryError::InvalidAddress(addr))),
        }
    }

    fn write(&mut self, addr: u16, byte: u8) -> crate::error::Result<()> {
        match addr {
            0x8000..=0xFFFF => {
                self.prg_rom[(addr & 0x3FFF) as usize] = byte;
                Ok(())
            }
            _ => Err(Box::new(MemoryError::InvalidAddress(addr))),
        }
    }
}


pub struct Nrom256 {
    prg_rom: [u8; 32 * 1024],
    chr_rom: [u8; 8 * 1024],
}

impl MemoryDevice for Nrom256 {
    fn name(&self) -> String { "NROM-256".into() }

    fn read(&self, addr: u16) -> crate::error::Result<u8> {
        match addr {
            0x8000..=0xFFFF => Ok(self.prg_rom[(addr & 0x7FFF) as usize]),
            _ => Err(Box::new(MemoryError::InvalidAddress(addr))),
        }
    }

    fn write(&mut self, addr: u16, byte: u8) -> crate::error::Result<()> {
        match addr {
            0x8000..=0xFFFF => {
                self.prg_rom[(addr & 0x7FFF) as usize] = byte;
                Ok(())
            }
            _ => Err(Box::new(MemoryError::InvalidAddress(addr))),
        }
    }
}

pub fn build_nrom_cart(prg_rom: &[u8], chr_rom: &[u8]) -> Result<Cartridge> {
    if prg_rom.len() == 16*1024 {
        Ok(Box::new(Nrom128 {
            prg_rom: prg_rom.try_into().unwrap(),
            chr_rom: chr_rom.try_into().expect("Invalid CHR ROM size for NROM-128.")
        }))
    } else if prg_rom.len() == 32*1024{
        Ok(Box::new(Nrom256 {
            prg_rom: prg_rom.try_into().unwrap(),
            chr_rom: chr_rom.try_into().expect("Invalid CHR ROM size for NROM-256.")
        })) 
    } else {
        Err("Unsupported PRG ROM size for NROM mapper".into())
    }
}

#[cfg(test)]
mod nrom_tests {
    use super::build_nrom_cart;

    #[test]
    fn test_build() {
        let chr = vec![0u8; 8*1024];
        assert_eq!(build_nrom_cart(&vec![0u8; 16*1024], &chr).unwrap().name(), "NROM-128");
        assert_eq!(build_nrom_cart(&vec![0u8; 32*1024], &chr).unwrap().name(), "NROM-256");

        build_nrom_cart(&vec![0u8; 8*1024], &chr).unwrap_err();
        build_nrom_cart(&vec![0u8; 64*1024], &chr).unwrap_err();
    }

    #[test]
    fn test_128() {
        let mut prg = vec![0u8; 16*1024];
        prg[0x17] = 0x42;
        let chr = vec![0u8; 8*1024];
        let mut cart = build_nrom_cart(&prg, &chr).unwrap();
        assert_eq!(cart.read(0x8017).unwrap(), 0x42);
        assert_eq!(cart.read(0xc017).unwrap(), 0x42);
        cart.write(0x9018, 0xff).unwrap();
        assert_eq!(cart.read(0xd018).unwrap(), 0xff);
    }

    #[test]
    fn test_256() {
        let mut prg = vec![0u8; 32*1024];
        prg[0x17] = 0x42;
        let chr = vec![0u8; 8*1024];
        let mut cart = build_nrom_cart(&prg, &chr).unwrap();
        assert_eq!(cart.read(0x8017).unwrap(), 0x42);
        assert_eq!(cart.read(0xC017).unwrap(), 0x00);
        cart.write(0x9018, 0xff).unwrap();
        assert_eq!(cart.read(0x9018).unwrap(), 0xff);
        assert_eq!(cart.read(0xD018).unwrap(), 0x00);
    }
}