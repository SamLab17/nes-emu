use super::cart::{Cart, Cartridge, ppu_inv_addr, ppu_rd_only};

use crate::error::Result;
use crate::mem::device::{MemoryDevice, MemoryError, inv_addr, rd_only};

#[derive(Debug)]
pub struct Nrom128 {
    prg_rom: [u8; 16 * 1024],
    prg_ram: [u8; 8 * 1024],
    chr_rom: [u8; 8 * 1024],
}

impl MemoryDevice for Nrom128 {
    fn name(&self) -> String { "NROM-128".into() }

    fn read(&mut self, addr: u16) -> Result<u8> {
        match addr {
            0x6000..=0x7FFF => Ok(self.prg_ram[((addr - 0x6000) as usize)]),
            0x8000..=0xFFFF => Ok(self.prg_rom[(addr & 0x3FFF) as usize]),
            _ => Err(inv_addr(addr)),
        }
    }

    fn write(&mut self, addr: u16, byte: u8) -> Result<()> {
        match addr {
            0x6000..=0x7FFF => Ok(self.prg_ram[((addr - 0x6000) as usize)] = byte),
            0x8000..=0xFFFF => Err(rd_only(addr)),
            _ => Err(inv_addr(addr)),
        }
    }
}

impl Cart for Nrom128 {
    fn ppu_read(&self, addr: u16, vram: &[u8]) -> Result<u8> {
        match addr {
            0x0000..=0x1FFF => Ok(self.chr_rom[addr as usize]),
            0x2000..=0x3EFF => Ok(vram[(addr & 0xFFF) as usize]),
            _ => Err(ppu_inv_addr(addr))
        }
    }

    fn ppu_write(&mut self, addr: u16, byte: u8, vram: &mut [u8]) -> Result<()> {
        match addr {
            0x0000..=0x1FFF => Ok(self.chr_rom[addr as usize] = byte),
            0x2000..=0x3EFF => Ok(vram[(addr & 0xFFF) as usize] = byte),
            _ => Err(ppu_inv_addr(addr))
        }
    } 
}


pub struct Nrom256 {
    prg_rom: [u8; 32 * 1024],
    prg_ram: [u8; 8 * 1024],
    chr_rom: [u8; 8 * 1024],
}

impl MemoryDevice for Nrom256 {
    fn name(&self) -> String { "NROM-256".into() }

    fn read(&mut self, addr: u16) -> Result<u8> {
        match addr {
            0x6000..=0x7FFF => Ok(self.prg_ram[((addr - 0x6000) as usize)]),
            0x8000..=0xFFFF => Ok(self.prg_rom[(addr & 0x7FFF) as usize]),
            _ => Err(Box::new(MemoryError::InvalidAddress(addr))),
        }
    }

    fn write(&mut self, addr: u16, byte: u8) -> Result<()> {
        match addr {
            0x6000..=0x7FFF => Ok(self.prg_ram[((addr - 0x6000) as usize)] = byte),
            0x8000..=0xFFFF => {
                self.prg_rom[(addr & 0x7FFF) as usize] = byte;
                Ok(())
            }
            _ => Err(inv_addr(addr)),
        }
    }
}

impl Cart for Nrom256 {
    fn ppu_read(&self, addr: u16, vram: &[u8]) -> Result<u8> {
        match addr {
            0x0000..=0x1FFF => Ok(self.chr_rom[addr as usize]),
            0x2000..=0x3EFF => Ok(vram[(addr & 0xFFF) as usize]),
            _ => Err(ppu_inv_addr(addr))
        }
    }

    fn ppu_write(&mut self, addr: u16, byte: u8, vram: &mut [u8]) -> Result<()> {
        match addr {
            0x0000..=0x1FFF => Ok(self.chr_rom[addr as usize] = byte),
            0x2000..=0x3EFF => Ok(vram[(addr & 0xFFF) as usize] = byte),
            _ => Err(ppu_inv_addr(addr))
        }
    } 
}

pub fn build_nrom_cart(prg_rom: &[u8], chr_rom: &[u8]) -> Result<Cartridge> {
    if prg_rom.len() == 16*1024 {
        Ok(Box::new(Nrom128 {
            prg_rom: prg_rom.try_into().unwrap(),
            prg_ram: [0; 8*1024],
            chr_rom: chr_rom.try_into().expect("Invalid CHR ROM size for NROM-128.")
        }))
    } else if prg_rom.len() == 32*1024{
        Ok(Box::new(Nrom256 {
            prg_rom: prg_rom.try_into().unwrap(),
            prg_ram: [0; 8*1024],
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
        prg[0x1018] = 0xff;
        let chr = vec![0u8; 8*1024];
        let mut cart = build_nrom_cart(&prg, &chr).unwrap();
        assert_eq!(cart.read(0x8017).unwrap(), 0x42);
        assert_eq!(cart.read(0xc017).unwrap(), 0x42);
        assert_eq!(cart.read(0xd018).unwrap(), 0xff);
    }

    #[test]
    fn test_256() {
        let mut prg = vec![0u8; 32*1024];
        prg[0x17] = 0x42;
        prg[0x1018] = 0xff;
        let chr = vec![0u8; 8*1024];
        let mut cart = build_nrom_cart(&prg, &chr).unwrap();
        assert_eq!(cart.read(0x8017).unwrap(), 0x42);
        assert_eq!(cart.read(0xC017).unwrap(), 0x00);
        assert_eq!(cart.read(0x9018).unwrap(), 0xff);
        assert_eq!(cart.read(0xD018).unwrap(), 0x00);
    }
}