use bit::BitIndex;

use crate::cart::cart::Cartridge;
use crate::mem::bus::MemoryBus;
use crate::mem::device::{MemoryDevice, MemoryError, inv_addr, rd_only, wr_only};
use crate::error::Result;

use super::colors::{load_color_map, ColorMap, load_default_color_map};

#[derive(Default, Debug)]
pub struct PpuReg {
    control: u8,
    mask: u8,
    status: u8,
    oam_addr: u8,
    oam_data: u8,
    ppu_scroll: u16,
    ppu_addr: u16,
    ppu_data: u8,
    oam_dma: u8
}

impl PpuReg {
    fn base_nt_addr(&self) -> u8 {self.control.bit_range(0..2)}
    fn vram_addr_incr(&self) -> bool { self.control.bit(2) }
    fn sprite_table_addr(&self) -> bool { self.control.bit(3) }
    fn bg_table_addr(&self) -> bool { self.control.bit(4) }
    fn sprite_size(&self) -> bool { self.control.bit(5) }
    fn ppu_master_slave(&self) -> bool { self.control.bit(6) }
    fn nmi_int_vblank(&self) -> bool { self.control.bit(7) }
}

pub struct PpuBuilder {
    palette_file: Option<String>
}

impl PpuBuilder {

    pub fn new() -> Self {
        PpuBuilder { palette_file: None }
    }

    pub fn with_palette(mut self, pal_file: String) -> Self {
        self.palette_file = Some(pal_file);
        self
    }

    pub fn build(self) -> Result<Ppu> {
        Ok(Ppu {
            color_map: load_color_map(self.palette_file.as_deref())?,
            ..Ppu::default()
        })
    }
}

pub struct Ppu {
    color_map: ColorMap,
    vram: [u8; 1024 * 2],
    palettes: [u8; 256],
    // Memory-mapped registers
    reg: PpuReg,
    // Internal registers 
    v: u16,
    t: u16,
    x: u8,
    w: bool
}

impl Ppu {
    // I honestly don't know why i'm allowed to have another "read" method
    // for the Ppu type but oh well...
    fn read(&self, cart: &Cartridge, addr: u16) -> Result<u8> {
        match addr {
            0x0000..=0x3EFF => cart.ppu_read(addr, &self.vram),
            0x3F00..=0x3FFF => Ok(self.palettes[(addr & 0x1F) as usize]),
            _ => Err(inv_addr(addr))
        }
    }

    fn write(&mut self, cart: &mut Cartridge, addr: u16, byte: u8) -> Result<()> {
        match addr {
            0x0000..=0x3EFF => cart.ppu_write(addr, byte, &mut self.vram),
            0x3F00..=0x3FFF => { self.palettes[(addr & 0x1F) as usize] = byte; Ok(()) }
            _ => Err(inv_addr(addr))
        }
    }

    pub fn run(&mut self, n_cycles: u32, cart: &mut Cartridge, bus: &MemoryBus) {
        // Cartridge is needed to access character ROM
        // Bus is needed to perform DMA from CPU to PPU
        todo!("PPU run")
    }
}

impl Default for Ppu {
    fn default() -> Self {
        Self { 
            color_map: load_default_color_map(),
            vram: [0u8; 1024*2],
            palettes: [0u8; 256],
            reg: PpuReg::default(),
            v: 0,
            t: 0,
            x: 0,
            w: false
         }
    }
}

impl MemoryDevice for Ppu {

    fn name(&self) -> String { "PPU".into() }

    fn read(&mut self, addr: u16) -> Result<u8> {
        if addr < 0x2000 || (addr > 0x3FFF && addr != 0x4014) {
            return Err(inv_addr(addr))
        } else if addr == 0x4014 {
            Err(wr_only(addr))
        } else {
            assert!(addr >= 0x2000 && addr <= 0x3FFF);
            match addr & 0x7 {
                0 => Err(wr_only(addr)),
                1 => Err(wr_only(addr)),
                2 => Ok(self.reg.status),
                3 => Err(wr_only(addr)),
                4 => Ok(self.reg.oam_data),
                5 => Err(wr_only(addr)),
                6 => Err(wr_only(addr)),
                7 => {
                    if self.reg.vram_addr_incr() {
                        self.v += 1;
                    } else {
                        self.v += 32;
                    }
                    self.v &= 0x7FFF;
                    Ok(self.vram[(self.v & 0x3FFF) as usize])
                },
                _ => panic!("impossible")
            }
        }
    }

    fn write(&mut self, addr: u16, byte: u8) -> Result<()> {
        if addr < 0x2000 || (addr > 0x3FFF && addr != 0x4014) {
            return Err(inv_addr(addr))
        } else if addr == 0x4014 {
            Err(wr_only(addr))
        } else {
            assert!(addr >= 0x2000 && addr <= 0x3FFF);
            match addr & 0x7 {
                0 => {
                    self.reg.control = byte;
                    self.t.set_bit_range(11..13, byte.bit_range(0..2).into());
                    Ok(())
                },
                1 => Ok(self.reg.mask = byte),
                2 => Err(rd_only(addr)),
                3 => Ok(self.reg.oam_addr = byte),
                4 => {
                    todo!("Increment oam_addr, and actually perform write?");
                    Ok(self.reg.oam_data = byte)
                },
                5 => {
                    self.reg.ppu_scroll <<= 8;
                    self.reg.ppu_scroll |= byte as u16;
                    Ok(())
                },
                6 => {
                    self.reg.ppu_addr <<= 8;
                    self.reg.ppu_addr |= byte as u16;
                    Ok(())
                },
                7 => {
                    if self.reg.vram_addr_incr() {
                        self.v += 1;
                    } else {
                        self.v += 32;
                    }
                    self.v &= 0x7FFF;
                    Ok(self.vram[(self.v & 0x3FFF) as usize] = byte)
                },
                _ => panic!("impossible")
            }
        }
    }
}