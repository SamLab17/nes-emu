use bit::BitIndex;

use super::cart::{ppu_inv_addr, ppu_rd_only, Cart, Cartridge};

use crate::error::Result;
use crate::ines::parse::MirrorType;
use crate::mem::device::{inv_addr, rd_only, MemoryDevice, MemoryError};

#[derive(Debug)]
pub struct Nrom128 {
    prg_rom: [u8; 16 * 1024],
    prg_ram: [u8; 8 * 1024],
    chr_rom: [u8; 8 * 1024],
    mirroring: MirrorType,
}

impl MemoryDevice for Nrom128 {
    fn name(&self) -> String {
        "NROM-128".into()
    }

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

/*
Nametable numbers:
          |
     00   |   01
  -----------------
     10  |   11
         |

Nametable Address:
NNOO OOOO OOOO
N = nametable number
O = offset
*/
fn nametable_addr(mut addr: u16, mirror_type: MirrorType) -> u16 {
    addr &= 0xFFF;
    // let nt_select = addr.bit_range(10..12);
    match mirror_type {
        MirrorType::Vertical => ((addr.bit(10) as u16) << 10) | (addr & 0x3FF),
        MirrorType::Horizontal => ((addr.bit(11) as u16) << 10) | (addr & 0x3FF),
    }
}

impl Cart for Nrom128 {
    fn ppu_read(&self, addr: u16, vram: &[u8]) -> Result<u8> {
        match addr {
            0x0000..=0x1FFF => Ok(self.chr_rom[addr as usize]),
            0x2000..=0x3EFF => Ok(vram[nametable_addr(addr, self.mirroring) as usize]),
            _ => Err(ppu_inv_addr(addr)),
        }
    }

    fn ppu_write(&mut self, addr: u16, byte: u8, vram: &mut [u8]) -> Result<()> {
        match addr {
            0x0000..=0x1FFF => Ok(self.chr_rom[addr as usize] = byte),
            0x2000..=0x3EFF => Ok(vram[nametable_addr(addr, self.mirroring) as usize] = byte),
            _ => Err(ppu_inv_addr(addr)),
        }
    }
}

pub struct Nrom256 {
    prg_rom: [u8; 32 * 1024],
    prg_ram: [u8; 8 * 1024],
    chr_rom: [u8; 8 * 1024],
    mirroring: MirrorType,
}

impl MemoryDevice for Nrom256 {
    fn name(&self) -> String {
        "NROM-256".into()
    }

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
            0x2000..=0x3EFF => Ok(vram[nametable_addr(addr, self.mirroring) as usize]),
            _ => Err(ppu_inv_addr(addr)),
        }
    }

    fn ppu_write(&mut self, addr: u16, byte: u8, vram: &mut [u8]) -> Result<()> {
        match addr {
            0x0000..=0x1FFF => Ok(self.chr_rom[addr as usize] = byte),
            0x2000..=0x3EFF => Ok(vram[nametable_addr(addr, self.mirroring) as usize] = byte),
            _ => Err(ppu_inv_addr(addr)),
        }
    }
}

pub fn build_nrom_cart(prg_rom: &[u8], chr_rom: &[u8], mirroring: MirrorType) -> Result<Cartridge> {
    if prg_rom.len() == 16 * 1024 {
        Ok(Box::new(Nrom128 {
            prg_rom: prg_rom.try_into().unwrap(),
            prg_ram: [0; 8 * 1024],
            chr_rom: chr_rom
                .try_into()
                .expect("Invalid CHR ROM size for NROM-128."),
            mirroring,
        }))
    } else if prg_rom.len() == 32 * 1024 {
        Ok(Box::new(Nrom256 {
            prg_rom: prg_rom.try_into().unwrap(),
            prg_ram: [0; 8 * 1024],
            chr_rom: chr_rom
                .try_into()
                .expect("Invalid CHR ROM size for NROM-256."),
            mirroring,
        }))
    } else {
        Err("Unsupported PRG ROM size for NROM mapper".into())
    }
}

#[cfg(test)]
mod nrom_tests {
    use super::build_nrom_cart;
    use crate::{cart::nrom::nametable_addr, ines::parse::MirrorType};

    #[test]
    fn test_build() {
        let chr = vec![0u8; 8 * 1024];
        assert_eq!(
            build_nrom_cart(&vec![0u8; 16 * 1024], &chr, MirrorType::Horizontal)
                .unwrap()
                .name(),
            "NROM-128"
        );
        assert_eq!(
            build_nrom_cart(&vec![0u8; 32 * 1024], &chr, MirrorType::Horizontal)
                .unwrap()
                .name(),
            "NROM-256"
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

    #[test]
    fn test_mirroring() {
        assert_eq!(nametable_addr(0x2800, MirrorType::Vertical), 0x000);
        assert_eq!(nametable_addr(0x2842, MirrorType::Vertical), 0x042);
        assert_eq!(nametable_addr(0x2917, MirrorType::Vertical), 0x117);

        assert_eq!(nametable_addr(0x2C00, MirrorType::Vertical), 0x400);
        assert_eq!(nametable_addr(0x2C42, MirrorType::Vertical), 0x442);
        assert_eq!(nametable_addr(0x2D17, MirrorType::Vertical), 0x517);

        assert_eq!(nametable_addr(0x2400, MirrorType::Horizontal), 0x000);
        assert_eq!(nametable_addr(0x2442, MirrorType::Horizontal), 0x042);
        assert_eq!(nametable_addr(0x2517, MirrorType::Horizontal), 0x117);

        assert_eq!(nametable_addr(0x2C00, MirrorType::Horizontal), 0x400);
        assert_eq!(nametable_addr(0x2C42, MirrorType::Horizontal), 0x442);
        assert_eq!(nametable_addr(0x2D17, MirrorType::Horizontal), 0x517);
    }
}
