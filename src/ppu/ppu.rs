use std::cell::RefCell;
use std::rc::Rc;

// use bit::BitIndex;
use bitfield::bitfield;

use crate::cart::cart::Cartridge;
use crate::cart::mock::mock_cart;
use crate::mem::bus::{MemoryBus};
use crate::mem::device::{MemoryDevice, MemoryError, inv_addr, rd_only, wr_only};
use crate::error::Result;
use crate::mem::ram::Ram;
use rand::Rng;
use sdl2::pixels::Color;


use super::colors::{load_color_map, ColorMap, load_default_color_map};

pub type Frame = Box<[[Color; 256]; 240]>;

bitfield! {
    #[derive(Debug, Default)]
    struct PpuControl(u8);
    u8;
    get_nametable_addr, set_nametable_addr: 1, 0;
    get_vram_inc, set_vram_inc: 2;
    get_sprite_table_addr, set_sprite_table_addr: 3;
    get_bg_table_addr, set_bg_table_addr: 4;
    get_sprite_size, set_sprite_size: 5;
    get_ppu_select, set_ppu_select: 6;
    get_nmi_toggle, set_nmi_toggle: 7;
}

bitfield! {
    #[derive(Debug, Default)]
    struct PpuMask(u8);
    u8;
    get_greyscale, set_greyscale: 0;
    get_show_bg_left, set_show_bg_left: 1;
    get_show_sprits_left, set_show_sprites_left: 2;
    get_show_bg, set_show_bg : 3;
    get_show_sprites, set_show_sprites: 4;
    get_emph_red, set_emph_red: 5;
    get_emph_green, set_emph_green: 6;
    get_emph_blue, set_emph_blue: 7;
}

bitfield! {
    #[derive(Debug, Default)]
    struct PpuStatus(u8);
    u8;
    get_open_bus, set_open_bus : 0, 4;
    get_sprite_overflow, set_sprite_overflow: 5;
    get_sprite_zero_hit, set_sprite_zero_hit: 6;
    get_vblank_start, set_vblank_start: 7;
}



#[derive(Default, Debug)]
pub struct PpuReg {
    control: PpuControl,
    mask: PpuMask,
    status: PpuStatus,
    oam_addr: u8,
    oam_data: u8,
    ppu_scroll: u16,
    ppu_addr: u16,
    // Whether we're writing to the upper or lower byte of ppu_addr
    ppu_addr_latch : bool,
    ppu_data: u8,
    ppu_data_buffer: u8,
    oam_dma: u8
}

impl PpuReg {
}

pub struct PpuBuilder {
    palette_file: Option<String>,
    cart: Rc<RefCell<Cartridge>>
}

impl PpuBuilder {

    pub fn new(cart: Rc<RefCell<Cartridge>>) -> Self {
        PpuBuilder { palette_file: None, cart }
    }

    pub fn with_palette(mut self, pal_file: String) -> Self {
        self.palette_file = Some(pal_file);
        self
    }

    pub fn build(self) -> Result<Ppu> {
        Ok(Ppu {
            color_map: load_color_map(self.palette_file.as_deref())?,
            cart: self.cart,
            vram: [0u8; 1024*2],
            palettes: [0u8; 256],
            reg: PpuReg::default(),
            v: 0,
            t: 0,
            x: 0,
            w: false,
            cycle: 0,
            scanline: 0,
            buffer: Box::new([[Color::BLACK; 256]; 240])
        })
    }
}

pub struct Ppu {
    pub buffer: Frame,
    cart: Rc<RefCell<Cartridge>>,
    color_map: ColorMap,
    vram: [u8; 1024 * 2],
    palettes: [u8; 256],
    // Memory-mapped registers
    reg: PpuReg,
    // Internal registers 
    v: u16,
    t: u16,
    x: u8,
    w: bool,
    cycle: u64,
    scanline: i32,
}

impl Ppu {
    // I honestly don't know why i'm allowed to have another "read" method
    // for the Ppu type but oh well...
    fn ppu_read(&self, addr: u16) -> Result<u8> {
        match addr {
            0x0000..=0x3EFF => self.cart.borrow_mut().ppu_read(addr, &self.vram),
            0x3F00..=0x3FFF => Ok(self.palettes[(addr & 0x1F) as usize]),
            _ => Err(inv_addr(addr))
        }
    }

    fn ppu_write(&mut self, addr: u16, byte: u8) -> Result<()> {
        match addr {
            0x0000..=0x3EFF => self.cart.borrow_mut().ppu_write(addr, byte, &mut self.vram),
            0x3F00..=0x3FFF => { self.palettes[(addr & 0x1F) as usize] = byte; Ok(()) }
            _ => Err(inv_addr(addr))
        }
    }

    pub fn reset(&mut self) -> Result<()> {
        todo!();
    }

    pub fn tick(&mut self, cpu_ram: &mut Ram) -> Result<Option<Frame>> {
        // Cartridge is needed to access character ROM
        // Bus is needed to perform DMA from CPU to PPU
        // let cart =&mut bus.cart;
        let mut ret_frame = None;
        // let cart = dma_bus.cart;

        // Let's render random noise
        // if self.scanline >= 0 && self.scanline < 240 && self.cycle < 256 {
        //     let row = self.scanline as usize;
        //     let col = self.cycle as usize;
        //     let mut rng = rand::thread_rng();
        //     let color = if rng.gen_bool(0.5) {
        //         self.color_map.get(&0x0f)
        //     } else {
        //         self.color_map.get(&0x30)
        //     }.expect("Color missing");
        //     // let color = self.color_map.get(&0x0F).unwrap();
        //     self.buffer[row][col] = *color;
        // }

        self.cycle += 1;
        if self.cycle >= 341 {
            self.cycle = 0;
            self.scanline += 1;
            if self.scanline >= 261 {
                self.scanline = -1;
                // frame is done
                ret_frame = Some(self.buffer.clone())
            }
        }

        Ok(ret_frame)
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
                2 => {
                   let ret = (self.reg.status.0 & 0xE0) | (self.reg.ppu_data_buffer & 0x1F);
                   self.reg.status.set_vblank_start(false);
                   self.reg.ppu_addr_latch = false;
                   Ok(ret)
                },
                3 => Err(wr_only(addr)),
                4 => Ok(self.reg.oam_data),
                5 => Err(wr_only(addr)),
                6 => Err(wr_only(addr)),
                7 => {
                    let mut data = self.reg.ppu_data_buffer;
                    self.reg.ppu_data_buffer = self.ppu_read(self.reg.ppu_addr)?;
                    if self.reg.ppu_addr > 0x3F00 {
                        data = self.reg.ppu_data_buffer;
                    }
                    Ok(data)
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
                    self.reg.control = PpuControl(byte);
                    // self.t.set_bit_range(11..13, byte.bit_range(0..2).into());
                    Ok(())
                },
                1 => Ok(self.reg.mask = PpuMask(byte)),
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
                    if self.reg.ppu_addr_latch {
                        // set lower byte
                        self.reg.ppu_addr = (self.reg.ppu_addr & 0xFF00) & (byte as u16);
                    } else {
                        // set high byte
                        self.reg.ppu_addr = (self.reg.ppu_addr & 0x00FF) & ((byte as u16) << 8);
                    }
                    self.reg.ppu_addr_latch = !self.reg.ppu_addr_latch;
                    
                    Ok(())
                },
                7 => {
                    if self.reg.control.get_vram_inc() {
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

#[cfg(test)]
mod ppu_test {

    use bitfield::bitfield;

    bitfield! {
        struct ControlReg(u8);
        u8;
        get_nametable_addr, set_nametable_addr: 1, 0;
        get_vram_inc, set_vram_inc: 2;
        get_sprite_table_addr, set_sprite_table_addr: 3;
        get_bg_table_addr, set_bg_table_addr: 4;
        get_sprite_size, set_sprite_size: 5;
        get_ppu_select, set_ppu_select: 6;
        get_nmi_toggle, set_nmi_toggle: 7;
    }

    #[test]
    fn bitfield_tests() {
        let mut c = ControlReg(0b11000110);
        assert_eq!(c.get_nametable_addr(), 0b10);
        assert_eq!(c.get_vram_inc(), true);

        c.set_nametable_addr(0b01);
        assert_eq!(c.get_nametable_addr(), 0b01);
        c = ControlReg(c.0 | 0b11);
        assert_eq!(c.get_nametable_addr(), 0b11);
        
        // Can we overwrite nearby fields? No
        c.set_nametable_addr(0xFF);
        assert_eq!(c.get_sprite_size(), false);
        c.set_sprite_size(true);
        c.set_nametable_addr(0x00);
        assert_eq!(c.get_sprite_size(), true);
    }
}