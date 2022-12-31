use super::cart::{nametable_addr, ppu_inv_addr, ppu_rd_only, Cart, Cartridge};

use crate::error::Result;
use crate::ines::parse::MirrorType;
use crate::mem::error::{inv_addr, rd_only};

#[derive(Debug)]
pub struct Nrom {
    prg_rom: Vec<u8>,
    prg_ram: [u8; 8 * 1024],
    chr_rom: Vec<u8>,
    chr_ram: [u8; 8 * 1024],
    mirroring: MirrorType,
    num_prg_banks: u8,
}

impl Cart for Nrom {
    fn name(&self) -> String {
        "NROM".into()
    }

    fn read(&mut self, addr: u16) -> Result<u8> {
        match addr {
            0x6000..=0x7FFF => Ok(self.prg_ram[((addr - 0x6000) as usize)]),
            0x8000..=0xFFFF => {
                let mask = if self.num_prg_banks == 1 {
                    0x3FFF
                } else {
                    0x7FFF
                };
                Ok(self.prg_rom[(addr & mask) as usize])
            }
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

    fn ppu_read(&self, addr: u16, vram: &[u8]) -> Result<u8> {
        match addr {
            0x0000..=0x1FFF => {
                if self.chr_rom.len() == 0 {
                    Ok(self.chr_ram[addr as usize])
                } else {
                    Ok(self.chr_rom[addr as usize])
                }
            }
            0x2000..=0x3EFF => Ok(vram[nametable_addr(addr, self.mirroring) as usize]),
            _ => Err(ppu_inv_addr(addr)),
        }
    }

    fn ppu_write(&mut self, addr: u16, byte: u8, vram: &mut [u8]) -> Result<()> {
        match addr {
            0x0000..=0x1FFF => {
                if self.chr_rom.len() == 0 {
                    Ok(self.chr_ram[addr as usize] = byte)
                } else {
                    Err(ppu_rd_only(addr))
                }
            }
            0x2000..=0x3EFF => Ok(vram[nametable_addr(addr, self.mirroring) as usize] = byte),
            _ => Err(ppu_inv_addr(addr)),
        }
    }
}

pub fn build_nrom_cart(prg_rom: &[u8], chr_rom: &[u8], mirroring: MirrorType) -> Result<Cartridge> {
    if prg_rom.len() == (16 * 1024) || prg_rom.len() == (32 * 1024) {
        let num_prg_banks = (prg_rom.len() / (16 * 1024)) as u8;
        Ok(Box::new(Nrom {
            prg_rom: prg_rom.to_vec(),
            prg_ram: [0; 8 * 1024],
            chr_rom: chr_rom.to_vec(),
            chr_ram: [0; 8 * 1024],
            mirroring,
            num_prg_banks,
        }))
    } else {
        Err("Unsupported PRG ROM size for NROM mapper".into())
    }
}

#[cfg(test)]
mod nrom_tests {
    use super::build_nrom_cart;
    use crate::ines::parse::MirrorType;

    #[test]
    fn test_build() {
        let chr = vec![0u8; 8 * 1024];
        assert_eq!(
            build_nrom_cart(&vec![0u8; 16 * 1024], &chr, MirrorType::Horizontal)
                .unwrap()
                .name(),
            "NROM"
        );
        assert_eq!(
            build_nrom_cart(&vec![0u8; 32 * 1024], &chr, MirrorType::Horizontal)
                .unwrap()
                .name(),
            "NROM"
        );

        build_nrom_cart(&vec![0u8; 8 * 1024], &chr, MirrorType::Horizontal).unwrap_err();
        build_nrom_cart(&vec![0u8; 64 * 1024], &chr, MirrorType::Horizontal).unwrap_err();
    }

    #[test]
    fn test_128() {
        let mut prg = vec![0u8; 16 * 1024];
        prg[0x17] = 0x42;
        prg[0x1018] = 0xff;
        let chr = vec![0u8; 8 * 1024];
        let mut cart = build_nrom_cart(&prg, &chr, MirrorType::Vertical).unwrap();
        assert_eq!(cart.read(0x8017).unwrap(), 0x42);
        assert_eq!(cart.read(0xc017).unwrap(), 0x42);
        assert_eq!(cart.read(0xd018).unwrap(), 0xff);
    }

    #[test]
    fn test_256() {
        let mut prg = vec![0u8; 32 * 1024];
        prg[0x17] = 0x42;
        prg[0x1018] = 0xff;
        let chr = vec![0u8; 8 * 1024];
        let mut cart = build_nrom_cart(&prg, &chr, MirrorType::Vertical).unwrap();
        assert_eq!(cart.read(0x8017).unwrap(), 0x42);
        assert_eq!(cart.read(0xC017).unwrap(), 0x00);
        assert_eq!(cart.read(0x9018).unwrap(), 0xff);
        assert_eq!(cart.read(0xD018).unwrap(), 0x00);
    }
}
