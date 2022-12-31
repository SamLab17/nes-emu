use std::cell::RefCell;
use std::rc::Rc;

use bitfield::bitfield;

use crate::cart::cart::Cartridge;
use crate::cpu::cpu::Interrupt;
use crate::error::Result;
use crate::mem::error::{inv_addr, rd_only, wr_only};
use sdl2::pixels::Color;

use super::colors::{load_color_map, ColorMap};

pub type Frame = Rc<RefCell<Box<[[Color; 256]; 240]>>>;
pub type PatternTable = Box<[[Color; 128]; 128]>;

fn set_low_byte(x: &mut u16, lsb: u8) {
    *x = (*x & 0xFF00) | (lsb as u16)
}

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

bitfield! {
    #[derive(Debug, Default)]
    struct PpuAddress(u16);
    u16;
    get_coarse_x, set_coarse_x: 4, 0;
    get_coarse_y, set_coarse_y: 9, 5;
    get_nametable_x, set_nametable_x: 10;
    get_nametable_y, set_nametable_y: 11;
    get_nametable, set_nametable: 11, 10;

    get_fine_y, set_fine_y: 14, 12;
    get_nametable_lookup_addr, _: 11, 0;
}

const NAMETABLE_OFFSET: u16 = 0x2000;
const ATTRIBUTE_TABLE_OFFSET: u16 = 0x23C0;
const PALETTES_OFFSET: u16 = 0x3F00;

#[derive(Default, Debug)]
pub struct PpuReg {
    control: PpuControl,
    mask: PpuMask,
    status: PpuStatus,
    oam_addr: u8,
    // Whether we're writing to the upper or lower byte of ppu_addr
    // aka "w" register
    ppu_addr_latch: bool,
    ppu_data: u8,
    ppu_data_buffer: u8,
    t_addr: PpuAddress,
    v_addr: PpuAddress,
    // aka "x" register
    fine_x: u8,
}

#[derive(Default, Debug)]
struct BackgroundState {
    tile_id: u8,
    tile_attribute: u8,
    tile_lsb: u8,
    tile_msb: u8,
    shift_pattern_lsb: u16,
    shift_pattern_msb: u16,
    shift_attribute_lsb: u16,
    shift_attribute_msb: u16,
}

#[derive(Debug, Clone)]
pub struct PendingSprite {
    sprite: OamSprite,
    row_offset: u8,
    col_offset: u8,
    is_sprite_zero: bool
}

#[derive(Default, Debug)]
struct ForegroundState {
    scanline_sprites: Vec<PendingSprite>,
    sprite_zero_hit: bool
}

bitfield! {
    #[derive(Debug, Default, Clone, Copy)]
    pub struct SpriteAttributes(u8);
    u8;
    get_palette, _: 1, 0;
    get_priority, _: 5;
    get_flip_horizontal, _ : 6;
    get_flip_vertical, _ : 7;
}

#[derive(Debug, Clone, Copy)]
pub struct OamSprite {
    pub y: u8,
    pub id: u8,
    pub attributes: SpriteAttributes,
    pub x: u8
}

// impl PpuReg {}

pub struct PpuBuilder {
    palette_file: Option<String>,
    cart: Rc<RefCell<Cartridge>>,
}

impl PpuBuilder {
    pub fn new(cart: Rc<RefCell<Cartridge>>) -> Self {
        PpuBuilder {
            palette_file: None,
            cart,
        }
    }

    // pub fn with_palette(mut self, pal_file: String) -> Self {
    //     self.palette_file = Some(pal_file);
    //     self
    // }

    pub fn build(self) -> Result<Ppu> {
        Ok(Ppu {
            color_map: load_color_map(self.palette_file.as_deref())?,
            cart: self.cart,
            vram: [0u8; 1024 * 2],
            oam: [0u8; 256],
            palettes: [0u8; 256],
            reg: PpuReg::default(),
            odd_frame: false,
            cycle: 0,
            scanline: 0,
            buffer: Rc::new(RefCell::new(Box::new([[Color::BLACK; 256]; 240]))),
            bg: BackgroundState::default(),
            fg: ForegroundState::default()
            // buffer: [[Color::BLACK; 256]; 240]
        })
    }
}

#[derive(Debug)]
pub struct Ppu {
    pub buffer: Frame,
    cart: Rc<RefCell<Cartridge>>,
    color_map: ColorMap,
    vram: [u8; 1024 * 2],
    oam: [u8; 256],
    palettes: [u8; 256],
    // Memory-mapped registers
    pub reg: PpuReg,
    odd_frame: bool,
    pub cycle: u64,
    pub scanline: i32,
    // Background rendering intermediates
   bg: BackgroundState,
   fg: ForegroundState
}

impl Ppu {
    pub fn read(&mut self, addr: u16) -> Result<u8> {
        if addr < 0x2000 || addr > 0x3FFF {
            return Err(inv_addr(addr));
        } else {
            assert!(addr >= 0x2000 && addr <= 0x3FFF);
            match addr & 0x7 {
                0 => Err(wr_only(addr)),
                1 => Err(wr_only(addr)),
                2 => {
                    // Status register
                    let ret = (self.reg.status.0 & 0xE0) | (self.reg.ppu_data_buffer & 0x1F);
                    self.reg.status.set_vblank_start(false);
                    self.reg.ppu_addr_latch = false;
                    //    println!("read status reg: {ret:X}");
                    Ok(ret)
                }
                3 => Err(wr_only(addr)),
                4 => Ok(self.oam[self.reg.oam_addr as usize]),
                5 => Err(wr_only(addr)),
                6 => Err(wr_only(addr)),
                7 => {
                    let mut data = self.reg.ppu_data_buffer;
                    self.reg.ppu_data_buffer = self.ppu_read(self.reg.v_addr.0)?;
                    if self.reg.v_addr.0 > 0x3F00 {
                        data = self.reg.ppu_data_buffer;
                    }
                    if self.reg.control.get_vram_inc() {
                        self.reg.v_addr.0 += 32
                    } else {
                        self.reg.v_addr.0 += 1
                    }
                    Ok(data)
                }
                _ => panic!("impossible"),
            }
        }
    }

    pub fn write(&mut self, addr: u16, byte: u8) -> Result<()> {
        if addr < 0x2000 || addr > 0x3FFF {
            return Err(inv_addr(addr));
        } else {
            match addr & 0x7 {
                0 => {
                    self.reg.control.0 = byte;
                    self.reg
                        .t_addr
                        .set_nametable(self.reg.control.get_nametable_addr().into());
                    Ok(())
                }
                1 => Ok(self.reg.mask.0 = byte),
                2 => Err(rd_only(addr)),
                3 => Ok(self.reg.oam_addr = byte),
                4 => {
                    self.oam[self.reg.oam_addr as usize] = byte;
                    Ok(())
                }
                5 => {
                    // println!("Write to scroll: 0x{:X}", byte);
                    // Scroll register
                    if self.reg.ppu_addr_latch {
                        self.reg.t_addr.set_coarse_y((byte >> 3) as u16);
                        self.reg.t_addr.set_fine_y((byte & 0x7) as u16);
                    } else {
                        self.reg.t_addr.set_coarse_x((byte >> 3) as u16);
                        self.reg.fine_x = byte & 0x7;
                    }
                    self.reg.ppu_addr_latch = !self.reg.ppu_addr_latch;
                    Ok(())
                }
                6 => {
                    if self.reg.ppu_addr_latch {
                        // set lower byte
                        set_low_byte(&mut self.reg.t_addr.0, byte);
                        self.reg.v_addr.0 = self.reg.t_addr.0;
                    } else {
                        // set high byte
                        self.reg.t_addr.0 =
                            (self.reg.t_addr.0 & 0x00FF) | (((byte as u16) & 0x3F) << 8);
                    }
                    self.reg.ppu_addr_latch = !self.reg.ppu_addr_latch;

                    Ok(())
                }
                7 => {
                    let ret = self.ppu_write(self.reg.v_addr.0, byte);
                    if self.reg.control.get_vram_inc() {
                        self.reg.v_addr.0 += 32
                    } else {
                        self.reg.v_addr.0 += 1
                    }
                    ret
                }
                _ => panic!("impossible"),
            }
        }
    }

    // Write a single byte into OAM memory
    pub fn oam_write(&mut self, offset: u8, data: u8) {
        self.oam[offset as usize] = data;
        if offset % 4 == 2 {
            // 3 middle bytes of byte 2 of a sprite should always read back as 0
            self.oam[offset as usize] &= 0xE3;
        }
    }

    // Read a sprite (4 bytes) from OAM memory
    pub fn oam_read(&self, index: u8) -> OamSprite {
        debug_assert!((index as usize) < self.oam.len() / 4);

        let off = (index as usize) << 2;
        OamSprite {
            y: self.oam[off], 
            id: self.oam[off + 1], 
            attributes: SpriteAttributes(self.oam[off + 2]), 
            x: self.oam[off + 3]
        }
    }

    fn map_palette_addr(&self, addr: u16) -> usize {
        let a = match addr {
            0x3F10 => 0x3F00,
            0x3F14 => 0x3F04,
            0x3F18 => 0x3F08,
            0x3F1C => 0x3F0C,
            _ => addr,
        } & 0x1F;
        (if self.reg.mask.get_greyscale() {
            a & 0x30
        } else {
            a & 0x3F
        }) as usize
    }

    fn ppu_read(&self, addr: u16) -> Result<u8> {
        match addr {
            0x0000..=0x3EFF => self.cart.borrow_mut().ppu_read(addr, &self.vram),
            0x3F00..=0x3FFF => Ok(self.palettes[self.map_palette_addr(addr)]),
            _ => Err(inv_addr(addr)),
        }
    }

    fn ppu_write(&mut self, addr: u16, byte: u8) -> Result<()> {
        match addr {
            0x0000..=0x3EFF => self.cart.borrow_mut().ppu_write(addr, byte, &mut self.vram),
            0x3F00..=0x3FFF => {
                self.palettes[self.map_palette_addr(addr)] = byte;
                Ok(())
            }
            _ => Err(inv_addr(addr)),
        }
    }

    pub fn reset(&mut self) -> Result<()> {
        self.reg.control = PpuControl(0);
        self.reg.mask = PpuMask(0);
        self.reg.fine_x = 0;
        self.reg.t_addr.0 = 0;
        self.reg.v_addr.0 = 0;
        self.reg.ppu_data = 0;
        self.odd_frame = false;
        self.cycle = 0;
        self.scanline = 0;
        self.reg.ppu_addr_latch = false;
        self.reg.ppu_data_buffer = 0;
        self.bg.tile_id = 0;
        self.bg.tile_lsb = 0;
        self.bg.tile_msb = 0;
        self.bg.shift_attribute_lsb = 0;
        self.bg.shift_attribute_msb = 0;
        self.bg.shift_pattern_lsb = 0;
        self.bg.shift_pattern_msb = 0;
        Ok(())
    }

    pub fn tick(&mut self) -> Result<(Option<Frame>, Option<Interrupt>)> {
        let mut ret_frame = None;
        let mut ret_int = None;

        if self.scanline >= -1 && self.scanline < 240 {
            // if self.scanline == 0 && self.cycle == 0 {
            //     self.cycle = 1;
            //     return Ok((None, None));
            // }
            if self.scanline == -1 && self.cycle == 1 {
                self.reg.status.set_vblank_start(false);
                self.reg.status.set_sprite_overflow(false);
                self.reg.status.set_sprite_zero_hit(false);
                self.fg.sprite_zero_hit = false;
            }
            if (2..258).contains(&self.cycle) || (321..338).contains(&self.cycle) {
                self.update_bg_shift();
                self.load_bg()?;
            }
            if (2..258).contains(&self.cycle) {
                self.update_sprites();
            }
            match self.cycle {
                256 => {
                    // End of scanline
                    self.inc_y();
                }
                257 => {
                    // Reset X
                    self.load_bg_shift();
                    self.copy_x();
                    if self.scanline >= 0 {
                        // Find sprites for the next scanline
                        self.find_sprites_for_scanline()
                    }
                }
                280..=304 if self.scanline == -1 => {
                    self.copy_y();
                }
                338 | 340 => {
                    self.bg.tile_id = self
                        .ppu_read(NAMETABLE_OFFSET | self.reg.v_addr.get_nametable_lookup_addr())?;
                }
                _ => (),
            }
            if self.scanline >= 0 && self.cycle == 340 {
                self.find_sprites_for_scanline()
            }
        }

        // if (241..261).contains(&self.scanline) {
        // Start of vblank period
        if self.scanline == 241 && self.cycle == 1 {
            self.reg.status.set_vblank_start(true);
            if self.reg.control.get_nmi_toggle() {
                ret_int = Some(Interrupt::NonMaskable);
            }
        }
        // }

        let mut pixel = 0;
        let mut palette = 0;
        let mut bg = true;
        if self.reg.mask.get_show_bg() {
            use bit::BitIndex;
            let bit_pos = 15 - (self.reg.fine_x as usize);
            let pixel0 = self.bg.shift_pattern_lsb.bit(bit_pos) as u8;
            let pixel1 = self.bg.shift_pattern_msb.bit(bit_pos) as u8;
            pixel = (pixel1 << 1) | pixel0;

            let pal0 = self.bg.shift_attribute_lsb.bit(bit_pos) as u8;
            let pal1 = self.bg.shift_attribute_msb.bit(bit_pos) as u8;
            palette = (pal1 << 1) | pal0;
            // color = self.get_color(palette, pixel, true)?;
            // println!("{palette:?} {pixel:?} {color:?}");
        }

        if self.reg.mask.get_show_sprites() {
            let mut sprite_pixel = 0;
            let mut sprite_palette = 0;
            let mut bg_priority = false;
            let mut sprite_zero = false;
            for idx in 0..self.fg.scanline_sprites.len() {
                let PendingSprite { sprite, row_offset, col_offset , is_sprite_zero} = self.fg.scanline_sprites[idx];
                if sprite.x == 0 && col_offset < 8 {
                    sprite_pixel = self.get_sprite_pixel(&sprite, row_offset, col_offset)?;
                    sprite_palette = sprite.attributes.get_palette();
                    bg_priority = sprite.attributes.get_priority();
                    sprite_zero = is_sprite_zero;
                    if sprite_pixel != 0 {
                        break;
                    }
                }
            }

            // Priority rules
            let bg_transparent = pixel == 0;
            let sprite_transparent = sprite_pixel == 0;
            if bg_transparent {
                pixel = sprite_pixel;
                palette = sprite_palette;
                bg = false;
            }
            else if !sprite_transparent && !bg_transparent {
                if !bg_priority {
                    // Sprite overwrites the background
                    pixel = sprite_pixel;
                    palette = sprite_palette;
                    bg = false;
                }
                // Check for sprite zero collision
                // if sprite_zero && self.reg.mask.get_show_bg() && self.reg.mask.get_show_sprites() && !self.fg.sprite_zero_hit {
                if sprite_zero && self.reg.mask.get_show_bg() && self.reg.mask.get_show_sprites() { 
                    if !self.reg.mask.get_show_bg_left() || !self.reg.mask.get_show_sprits_left() {
                        if (9..258).contains(&self.cycle) {
                            self.reg.status.set_sprite_zero_hit(true);
                            self.fg.sprite_zero_hit = true;
                            // println!("sprite zero hit");
                        } 
                    } else {
                        if (1..258).contains(&self.cycle) {
                            self.reg.status.set_sprite_zero_hit(true);
                            self.fg.sprite_zero_hit = true;
                            // println!("sprite zero hit");
                        }
                    }
                }
            }
        }

        if self.cycle > 0  && self.rendering_enabled() {
            let row = self.scanline as usize;
            let col = (self.cycle - 1) as usize;
            if row < self.buffer.borrow().len() && col < self.buffer.borrow()[row].len() {
                self.buffer.borrow_mut()[row][col] = self.get_color(palette, pixel, bg)?;
            }
        }

        self.cycle += 1;
        if self.cycle >= 341 {
            self.cycle = 0;
            self.scanline += 1;
            if self.scanline >= 261 {
                self.scanline = -1;
                // frame is done
                ret_frame = Some(self.buffer.clone());
                self.odd_frame = !self.odd_frame;
            }
        }

        Ok((ret_frame, ret_int))
    }

    fn rendering_enabled(&self) -> bool {
        self.reg.mask.get_show_bg() || self.reg.mask.get_show_sprites()
    }

    fn inc_x(&mut self) {
        if self.rendering_enabled() {
            let cx = self.reg.v_addr.get_coarse_x();
            self.reg.v_addr.set_coarse_x((cx + 1) % 32);
            if self.reg.v_addr.get_coarse_x() == 0 {
                // Wrapped around, go to next nametable
                self.reg
                    .v_addr
                    .set_nametable_x(!self.reg.v_addr.get_nametable_x());
            }
        }
    }

    fn inc_y(&mut self) {
        if self.rendering_enabled() {
            let fy = self.reg.v_addr.get_fine_y();
            let cy = self.reg.v_addr.get_coarse_y();
            self.reg.v_addr.set_fine_y((fy + 1) % 8);
            if self.reg.v_addr.get_fine_y() == 0 {
                self.reg.v_addr.set_coarse_y((cy + 1) % 30);
                if self.reg.v_addr.get_coarse_y() == 0 {
                    self.reg
                        .v_addr
                        .set_nametable_y(!self.reg.v_addr.get_nametable_y());
                }
            }
            // Clamp coarse_y just in case
            self.reg
                .v_addr
                .set_coarse_y(self.reg.v_addr.get_coarse_y().min(29));
        }
    }

    fn copy_x(&mut self) {
        if self.rendering_enabled() {
            self.reg
                .v_addr
                .set_nametable_x(self.reg.t_addr.get_nametable_x());
            self.reg.v_addr.set_coarse_x(self.reg.t_addr.get_coarse_x());
        }
    }

    fn copy_y(&mut self) {
        if self.rendering_enabled() {
            self.reg.v_addr.set_fine_y(self.reg.t_addr.get_fine_y());
            self.reg
                .v_addr
                .set_nametable_y(self.reg.t_addr.get_nametable_y());
            self.reg.v_addr.set_coarse_y(self.reg.t_addr.get_coarse_y());
        }
    }

    fn load_bg_shift(&mut self) {
        set_low_byte(&mut self.bg.shift_pattern_lsb, self.bg.tile_lsb);
        set_low_byte(&mut self.bg.shift_pattern_msb, self.bg.tile_msb);
        let attr_lo = if self.bg.tile_attribute & 0b1 != 0 {
            0xFF
        } else {
            0
        };
        let attr_hi = if self.bg.tile_attribute & 0b10 != 0 {
            0xFF
        } else {
            0
        };
        set_low_byte(&mut self.bg.shift_attribute_lsb, attr_lo);
        set_low_byte(&mut self.bg.shift_attribute_msb, attr_hi);
    }

    fn update_bg_shift(&mut self) {
        if self.reg.mask.get_show_bg() {
            self.bg.shift_pattern_lsb <<= 1;
            self.bg.shift_pattern_msb <<= 1;
            self.bg.shift_attribute_lsb <<= 1;
            self.bg.shift_attribute_msb <<= 1;
        }
    }

    fn load_bg(&mut self) -> Result<()> {
        let pattern_table_addr = if self.reg.control.get_bg_table_addr() {
            0x1000
        } else {
            0x0
        };

        // https://www.nesdev.org/w/images/default/4/4f/Ppu.svg
        match (self.cycle - 1) % 8 {
            1 | 3 | 5 => (),
            0 => {
                self.load_bg_shift();
                // load nametable entry
                self.bg.tile_id = self.ppu_read(self.reg.v_addr.get_nametable_lookup_addr() | NAMETABLE_OFFSET)?;
            }
            2 => {
                // Attribute Address:
                // 01NN1111YYYXXX
                // X, Y are the top 3 bits of the coarse_x and coarse_y V addr registers

                // load attribute for tile
                self.bg.tile_attribute = self.ppu_read(
                    ATTRIBUTE_TABLE_OFFSET |
                    (self.reg.v_addr.0 & 0xC00) | // Get nametable select
                    (self.reg.v_addr.get_coarse_x() >> 2) |
                    ((self.reg.v_addr.get_coarse_y() >> 2) << 3)
                )?;
                // Byte from attribute table consists of four pairs of two bits
                if self.reg.v_addr.get_coarse_y() & 0b10 != 0 {
                    // We're in the top of the quadrant
                    self.bg.tile_attribute >>= 4;
                }
                if self.reg.v_addr.get_coarse_x() & 0b10 != 0 {
                    // We're in the right of the quadrant
                    self.bg.tile_attribute >>= 2;
                }
                // We only care about the bottom two bits
                self.bg.tile_attribute &= 0b11;
            }
            4 => {
                // load lsb of tile data
                self.bg.tile_lsb = self.ppu_read(
                    pattern_table_addr
                    + ((self.bg.tile_id as u16) << 4)
                    + self.reg.v_addr.get_fine_y()
                )?;
            }
            6 => {
                // load msb of tile data
                self.bg.tile_msb = self.ppu_read(
                    pattern_table_addr
                        + ((self.bg.tile_id as u16) << 4)
                        + self.reg.v_addr.get_fine_y()
                        + 8,
                )?;
            }
            7 => {
                // update registers
                self.inc_x();
            }
            _ => panic!("impossible"),
        }

        Ok(())
    }

    fn get_color(&mut self, mut palette_idx: u8, mut pixel: u8, bg: bool) -> Result<Color> {
        // (pixel is an index into the palette)
        palette_idx &= 0b11;
        pixel &= 0b11;
        if pixel == 0 {
            palette_idx = 0;
        }
        let bg_select = !bg as u8;
        let idx = (bg_select << 4) | (palette_idx << 2) | pixel;
        let addr = PALETTES_OFFSET | (idx as u16);
        let color = self.ppu_read(addr)?;
        match self.color_map.get(&color) {
            Some(c) => Ok(*c),
            None => Err("Invalid color".into()),
        }
    }

    fn find_sprites_for_scanline(&mut self) {
        assert!(self.scanline >= 0);

        let sprite_height = if self.reg.control.get_sprite_size() {
            16
        } else {
            8
        };

        self.fg.scanline_sprites.clear();
        
        for idx in 0..64 {
            let sprite = self.oam_read(idx);
            // Notice that we use the current scanline number here but are preparing
            // sprites for the _next_ scanline. This is intentional and sprites in OAM memory
            // have their y position offset by 1 because of this.
            let diff = (self.scanline as i16) - (sprite.y as i16);
            if diff >= 0 && diff < sprite_height {
                if self.fg.scanline_sprites.len() < 8 {
                    let ps = PendingSprite { sprite, row_offset: diff as u8, col_offset: 0, is_sprite_zero: idx == 0 } ;
                    self.fg.scanline_sprites.push(ps);
                } else {
                    self.reg.status.set_sprite_overflow(true);
                    break;
                }
            }
        }
        assert!(self.fg.scanline_sprites.len() <= 8);
    }

    fn update_sprites(&mut self) {
        if self.reg.mask.get_show_sprites() {
            for PendingSprite { sprite, row_offset: _, col_offset , is_sprite_zero: _} in self.fg.scanline_sprites.iter_mut() {
                if sprite.x > 0 {
                    sprite.x -= 1;
                } else if *col_offset < 8 {
                    *col_offset += 1;
                }
            }
        }
    }

    fn get_sprite_pixel(&mut self, sprite: &OamSprite, mut r: u8, mut c: u8) -> Result<u8> {
        use bit::BitIndex;

        debug_assert!(c < 8);

        if self.reg.control.get_sprite_size() {
            // 8x16
            debug_assert!(r < 16);
            if !sprite.attributes.get_flip_horizontal() {
                // Because of the way we're indexing into the byte, we subtract from 7 when
                // _not_ flipped. (If flipped then we keep c as is)
                c = 7 - c;
            }
            if sprite.attributes.get_flip_vertical() {
                r = 15 - r;
            }

            let pattern_base = ((sprite.id & 1) as u16) << 12;

            let tile_addr_lo = pattern_base 
                                | ((sprite.id & 0xFE) as u16 * 16)      // Each pattern is 16 bytes
                                | (r as u16);                         // Row offset
            let tile_addr_hi = tile_addr_lo + 8;
            let lo = self.ppu_read(tile_addr_lo)?;
            let hi = self.ppu_read(tile_addr_hi)?;
            let pixel = ((hi.bit(c as usize) as u8) << 1) | (lo.bit(c as usize) as u8);
            Ok(pixel) 
        } else {
            // 8x8
            assert!(r < 8);
            let pattern_base = (self.reg.control.get_sprite_table_addr() as u16) << 12;

            if !sprite.attributes.get_flip_horizontal() {
                c = 7 - c;
            } 
            if sprite.attributes.get_flip_vertical() {
                r = 7 - r;
            }
            let tile_addr_lo = pattern_base 
                                | (sprite.id as u16 * 16) // Each pattern is 16 bytes
                                | (r as u16);           // Row offset
            let tile_addr_hi = tile_addr_lo + 8;
            let lo = self.ppu_read(tile_addr_lo)?;
            let hi = self.ppu_read(tile_addr_hi)?;
            let pixel = ((hi.bit(c as usize) as u8) << 1) | (lo.bit(c as usize) as u8);
            Ok(pixel)
        }
    }

    // returns the 4 background and 4 foreground palettes
    pub fn debug_palettes(&mut self) -> Vec<Vec<Color>> {
        let mut background: Vec<Vec<Color>> = (0..4)
            .map(|palette| {
                (0..4)
                    .map(|pixel| self.get_color(palette, pixel, true).unwrap_or(Color::BLACK))
                    .collect()
            })
            .collect();

        let foreground: Vec<Vec<Color>> = (0..4)
            .map(|palette| {
                (0..4)
                    .map(|pixel| {
                        self.get_color(palette, pixel, false)
                            .unwrap_or(Color::BLACK)
                    })
                    .collect()
            })
            .collect();

        background.extend(foreground.into_iter());
        background
    }

    pub fn debug_pattern_tables(
        &mut self,
        palette: u8,
        bg: bool,
    ) -> Result<(PatternTable, PatternTable)> {
        use bit::BitIndex;
        let mut pat0: PatternTable = Box::new([[Color::BLACK; 128]; 128]);
        let mut pat1: PatternTable = Box::new([[Color::BLACK; 128]; 128]);

        for tile in 0..256 {
            let lo_plane0_addr = tile * 16;
            let hi_plane0_addr = lo_plane0_addr + 8;
            let lo_plane1_addr = tile * 16 + 0x1000;
            let hi_plane1_addr = lo_plane1_addr + 8;

            let tile_x = (tile % 16) * 8;
            let tile_y = (tile / 16) * 8;

            for row in 0..8 {
                let lo0 = self.ppu_read(lo_plane0_addr + row)?;
                let hi0 = self.ppu_read(hi_plane0_addr + row)?;
                let lo1 = self.ppu_read(lo_plane1_addr + row)?;
                let hi1 = self.ppu_read(hi_plane1_addr + row)?;
                for col in 0..8u16 {
                    let pixel0 =
                        ((hi0.bit(7 - col as usize) as u8) << 1) | lo0.bit(7 - col as usize) as u8;
                    pat0[(tile_y + row) as usize][(tile_x + col) as usize] =
                        self.get_color(palette, pixel0, bg)?;
                    let pixel1 =
                        ((hi1.bit(7 - col as usize) as u8) << 1) | lo1.bit(7 - col as usize) as u8;
                    pat1[(tile_y + row) as usize][(tile_x + col) as usize] =
                        self.get_color(palette, pixel1, bg)?;
                }
            }
        }
        Ok((pat0, pat1))
    }
}

#[cfg(test)]
mod ppu_test {

    use bitfield::bitfield;

    use super::{PpuAddress, SpriteAttributes};

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

    #[test]
    fn ppu_addr_tests() {
        let mut a = PpuAddress(0);
        a.set_coarse_x(0x1F);
        assert_eq!(a.0, 0x1F);
        a.set_nametable(0x1);
        assert_eq!(a.0, 0b010000011111);
        a.set_coarse_y(0b10101);
        assert_eq!(a.0, 0b011010111111);
        a.set_fine_y(0b111);
        assert_eq!(a.0, 0b111011010111111);
        assert_eq!(a.get_nametable_lookup_addr(), 0b011010111111);
        a.0 = 0;
        assert_eq!(a.0, 0);
        assert_eq!(a.get_fine_y(), 0);
        a.0 = 0xFFFF;
        assert_eq!(a.get_nametable_lookup_addr(), a.0 & 0x0FFF);
    }

    #[test]
    fn sprite_attr_tests() {
        let attr = SpriteAttributes(0xE3);
        assert!(attr.get_flip_horizontal());
        assert!(attr.get_flip_vertical());
        let attr = SpriteAttributes(0);
        assert!(!attr.get_flip_horizontal());
        assert!(!attr.get_flip_vertical());
    }
}
