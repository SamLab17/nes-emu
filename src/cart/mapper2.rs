use super::cart::{Cartridge, ppu_inv_addr, nametable_addr, ppu_rd_only};

use crate::ines::parse::MirrorType;
use crate::error::Result;
use crate::mem::error::{rd_only, inv_addr};

use super::cart::Cart;


#[derive(Debug)]
pub struct Uxrom {
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
    chr_ram: [u8; 8 * 1024],
    bank_select: u8,
    mirror_type: MirrorType
}

impl Cart for Uxrom {
    fn name(&self) -> String {
        "UxROM".into()
    }

    fn read(&mut self, addr: u16) -> Result<u8> {
        match addr {
            0x6000..=0x7FFF => Err(inv_addr(addr)),
            0x8000..=0xBFFF => {
                let base = (self.bank_select as usize) * (16*1024);
                let offset =  (addr & 0x3FFF) as usize;
                Ok(self.prg_rom[base + offset]) 
            },
            0xC000..=0xFFFF => {
                // Hardwired to last bank
                let base = self.prg_rom.len() - (16 * 1024);
                let offset = (addr & 0x3FFF) as usize;
                Ok(self.prg_rom[base + offset])
            }
            _ => Err(inv_addr(addr)),
        }
    }

    fn write(&mut self, addr: u16, byte: u8) -> Result<()> {
        match addr {
            0x6000..=0x7FFF => Err(inv_addr(addr)),
            0x8000..=0xFFFF => {
                self.bank_select = byte & 0xF;
                Ok(())
            },
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
            0x2000..=0x3EFF => Ok(vram[nametable_addr(addr, self.mirror_type) as usize]),
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
            0x2000..=0x3EFF => Ok(vram[nametable_addr(addr, self.mirror_type) as usize] = byte),
            _ => Err(ppu_inv_addr(addr)),
        }
    }
}

pub fn build_uxrom(prg_rom: &[u8], chr_rom: &[u8], mirror_type: MirrorType) -> Result<Cartridge> {
    Ok(Box::new(Uxrom{
        prg_rom: prg_rom.to_vec(),
        chr_rom: chr_rom.to_vec(),
        chr_ram: [0; 8*1024],
        bank_select: 0,
        mirror_type,
    })
    )
}