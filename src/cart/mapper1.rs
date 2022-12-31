use bitfield::bitfield;

use crate::error::Result;
use crate::{ines::parse::MirrorType, mem::error::inv_addr};

use super::cart::{nametable_addr, ppu_inv_addr, ppu_rd_only, Cart, Cartridge};

bitfield! {
    #[derive(Debug, Default)]
    struct ControlReg(u8);
    u8;
    get_mirror, set_mirror: 1,0;
    get_prg_rom_bank_mode, set_prg_rom_bank_mode: 3,2;
    get_chr_rom_bank_mode, set_chr_rom_bank_mode: 4;
}

#[derive(Debug)]
pub struct Mmc1 {
    prg_ram: [u8; 8 * 1024],
    prg_rom: Vec<u8>, // up to 512K
    chr_rom: Vec<u8>,
    chr_ram: [u8; 8 * 1024],
    // Internal registers
    shift_reg: u8,
    control: ControlReg,
    chr_bank0: u8,
    chr_bank1: u8,
    prg_bank: u8,
    write_count: u8,
    prg_rom_bank_0_base: usize,
    prg_rom_bank_1_base: usize,
    chr_rom_bank_0_base: usize,
    chr_rom_bank_1_base: usize,
    mirror_type: MirrorType,
}

impl Mmc1 {
    fn update_base_addr(&mut self) {
        use MirrorType::*;
        self.mirror_type = match self.control.get_mirror() {
            0 => OneScreenLow,
            1 => OneScreenHigh,
            2 => Vertical,
            3 => Horizontal,
            _ => panic!("impossible"),
        };

        // Set CHR banks
        if self.control.get_chr_rom_bank_mode() == false {
            // 8K CHR ROM mode
            self.chr_rom_bank_0_base = ((self.chr_bank0 >> 1) as usize) * (8 * 1024);
            // bank 1 is just 4K offset from bank 0
            self.chr_rom_bank_1_base = self.chr_rom_bank_0_base + (4 * 1024);
        } else {
            // Two 4K banks
            self.chr_rom_bank_0_base = (self.chr_bank0 as usize) * (4 * 1024);
            self.chr_rom_bank_1_base = (self.chr_bank1 as usize) * (4 * 1024);
        }

        // Set PRG banks
        match self.control.get_prg_rom_bank_mode() {
            0 | 1 => {
                // One 32K bank
                self.prg_rom_bank_0_base = ((self.prg_bank as usize & 0xF) >> 1) * (32*1024);
                self.prg_rom_bank_1_base = self.prg_rom_bank_0_base + (16*1024);
            }
            2 => {
                // Fix first bank at 0
                self.prg_rom_bank_0_base = 0;
                // Switch 16K bank at 1
                self.prg_rom_bank_1_base = (self.prg_bank as usize & 0xF) * (16*1024);
            }
            3 => {
                // Switch 16K bank at 0
                self.prg_rom_bank_0_base = (self.prg_bank as usize & 0xF) * (16*1024);
                // Fix last bank at 1
                self.prg_rom_bank_1_base = (self.prg_rom.len() as usize) - (16*1024);
            }
            _ => panic!("impossible"),
        };
    }

    fn map_cpu_addr(&self, addr: u16) -> usize {
        let bank = addr & 0x4000;
        if bank == 0 {
            self.prg_rom_bank_0_base | (addr as usize & 0x3FFF)
        } else {
            self.prg_rom_bank_1_base | (addr as usize & 0x3FFF)
        }
    }

    fn map_chr_addr(&self, addr: u16) -> usize {
        let bank = addr & 0x1000;
        if bank == 0 {
            self.chr_rom_bank_0_base + (addr as usize & 0xFFF)
        } else {
            self.chr_rom_bank_1_base + (addr as usize & 0xFFF)
        }
    }
}

impl Cart for Mmc1 {
    fn name(&self) -> String {
        "MMC1".into()
    }

    fn read(&mut self, addr: u16) -> crate::error::Result<u8> {
        match addr {
            0x6000..=0x7FFF => Ok(self.prg_ram[((addr - 0x6000) as usize)]),
            0x8000..=0xFFFF => Ok(self.prg_rom[self.map_cpu_addr(addr)]),
            _ => Err(inv_addr(addr)),
        }
    }

    fn write(&mut self, addr: u16, byte: u8) -> crate::error::Result<()> {
        match addr {
            0x6000..=0x7FFF => Ok(self.prg_ram[((addr - 0x6000) as usize)] = byte),
            0x8000..=0xFFFF => {
                if byte & 0x80 != 0 {
                    self.write_count = 0;
                    self.shift_reg = 0;
                    self.control.set_prg_rom_bank_mode(0b11);
                    self.update_base_addr();
                } else {
                    self.shift_reg = ((byte & 1) << 4) | (self.shift_reg >> 1);
                    self.write_count += 1;
                    if self.write_count == 5 {
                        match (addr >> 13) & 3 {
                            0 => self.control = ControlReg(self.shift_reg),
                            1 => self.chr_bank0 = self.shift_reg,
                            2 => self.chr_bank1 = self.shift_reg,
                            3 => self.prg_bank = self.shift_reg,
                            _ => panic!("Impossible"),
                        };
                        self.write_count = 0;
                        self.shift_reg = 0;
                        self.update_base_addr();
                    }
                }
                // ROM is read-only, not actually written to.
                Ok(())
            }
            _ => Err(inv_addr(addr)),
        }
    }

    fn ppu_read(&self, addr: u16, vram: &[u8]) -> Result<u8> {
        match addr {
            0x0000..=0x1FFF => {
                if self.chr_rom.len() == 0 {
                    // use CHR RAM (only 8K, no bank switching)
                    Ok(self.chr_ram[self.map_chr_addr(addr)])
                } else {
                    Ok(self.chr_rom[self.map_chr_addr(addr)])
                }
            },
            0x2000..=0x3EFF => Ok(vram[nametable_addr(addr, self.mirror_type) as usize]),
            _ => Err(ppu_inv_addr(addr)),
        }
    }

    fn ppu_write(&mut self, addr: u16, byte: u8, vram: &mut [u8]) -> Result<()> {
        match addr {
            0x0000..=0x1FFF => {
                if self.chr_rom.len() == 0 {
                    // use CHR RAM (only 8K, no bank switching)
                    Ok(self.chr_ram[self.map_chr_addr(addr)] = byte)
                } else {
                    // CHR ROM not writeable
                    Err(ppu_rd_only(addr))
                }
            },
            0x2000..=0x3EFF => Ok(vram[nametable_addr(addr, self.mirror_type) as usize] = byte),
            _ => Err(ppu_inv_addr(addr)),
        }
    }
}

pub fn build_mmc1_cart(prg_rom: &[u8], chr_rom: &[u8]) -> Result<Cartridge> {
    let mut cart = Mmc1 {
        prg_ram: [0; 8 * 1024],
        prg_rom: prg_rom.to_vec(),
        chr_rom: chr_rom.to_vec(),
        chr_ram: [0; 8*1024],
        shift_reg: 0,
        control: ControlReg(0x0C),
        chr_bank0: 0,
        chr_bank1: 0,
        prg_bank: 0,
        write_count: 0,
        prg_rom_bank_0_base: 0,
        prg_rom_bank_1_base: 0,
        chr_rom_bank_0_base: 0,
        chr_rom_bank_1_base: 0,
        mirror_type: MirrorType::OneScreenLow,
    };
    cart.update_base_addr();
    Ok(Box::new(cart))
}
