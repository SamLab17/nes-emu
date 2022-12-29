use crate::cpu::cpu::Cpu;
use crate::cpu::isa::{Instr, Opcode, AddressingMode};
use crate::error::Result;
use nes_emu::ppu::ppu::Frame;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::{Rect, Point};
use sdl2::render::{Canvas, Texture};
use sdl2::surface::Surface;
use sdl2::video::Window;
use sdl2::{EventPump, Sdl};

use super::graphics::NesGraphics;

pub struct DebugGraphics {
    context: Sdl,
    canvas: Canvas<Window>,
    event_pump: EventPump,
    font_texture: Texture,
    character_rects: [Rect; 256],
    show_nametable_boundaries: bool,
    iscale: u32,
    curr_palette: u8,
}

impl NesGraphics for DebugGraphics {
    fn render_frame(&mut self, frame: Frame, cpu: &mut Cpu) -> Result<()> {
        self.canvas.set_draw_color(Color::BLACK);
        self.canvas.clear();

        // Draw NES graphics
        for r in 0..frame.len() {
            for c in 0..frame[r].len() {
                self.canvas.set_draw_color(frame[r][c]);
                self.canvas.fill_rect(Rect::new(
                    (c * (self.iscale as usize)).try_into().unwrap(),
                    (r * (self.iscale as usize)).try_into().unwrap(),
                    self.iscale,
                    self.iscale,
                ))?;
            }
        }

        if self.show_nametable_boundaries {
            self.canvas.set_draw_color(Color::RED);
            let s = self.iscale as i32;
            let w = Self::NES_WIDTH as i32;
            let h = Self::NES_HEIGHT as i32;
            for r in 0..30 {
                self.canvas.draw_line(Point::new(0, r*8*s), Point::new(w * s, r * 8 * s))?;
            }
            for c in 0..32 {
                self.canvas.draw_line(Point::new(c * 8 * s, 0), Point::new(c*8*s, h * s))?;
            }
        }

        // Draw Palette Tables
        let palettes = cpu.bus.ppu.debug_palettes();
        let (p0, p1) = cpu
            .bus
            .ppu
            .debug_pattern_tables(self.curr_palette % 4, self.curr_palette < 4)?;

        // println!("palette 0: {:?}", palettes[0]);
        // println!("pattern table: {:?}", p0);
        self.draw_pattern_table(
            &p1,
            2,
            self.width() as i32 - 256,
            self.height() as i32 - 256,
        )?;
        self.draw_pattern_table(
            &p0,
            2,
            self.width() as i32 - 512,
            self.height() as i32 - 256,
        )?;
        self.draw_palettes(&palettes)?;
        let pal_str = format!("Palette: {}", self.curr_palette);
        self.write_text(&pal_str, self.width() as i32 - 512, self.height() as i32 - 280, Some(2.0), Color::WHITE)?;

        let reg_str = format!("{}", cpu.reg);
        self.write_text(&reg_str, self.width() as i32 - 512, 0, Some(1.5), Color::WHITE)?;

        let (pc0, i0) = cpu.peek_next_instr(0).unwrap_or((0, Instr{op: Opcode::NOP, mode: AddressingMode::Implied}));
        let (pc1, i1) = cpu.peek_next_instr(1).unwrap_or((0, Instr{op: Opcode::NOP, mode: AddressingMode::Implied}));
        let (pc2, i2) = cpu.peek_next_instr(2).unwrap_or((0, Instr{op: Opcode::NOP, mode: AddressingMode::Implied}));
        let i_str = format!("{:04X}: {}\n{:04X}: {}\n{:04X}: {}", pc0, i0, pc1, i1, pc2, i2);
        self.write_text(&i_str, self.width() as i32 - 512, 32, Some(2.0), Color::WHITE)?;

        self.canvas.present();
        Ok(())
    }

    fn events(&mut self) -> Vec<Event> {
        let events = self.event_pump.poll_iter().collect::<Vec<Event>>();
        for e in events.iter() {
            match e {
                Event::KeyDown { keycode: Some(Keycode::P), .. } => {
                    self.curr_palette = (self.curr_palette + 1) % 8;
                },
                Event::KeyDown { keycode: Some(Keycode::N), .. } => {
                    self.show_nametable_boundaries = !self.show_nametable_boundaries;
                }
                _ => (),
            }
        }
        events
    }

    fn performance_frequency(&self) -> Result<u64> {
        Ok(self.context.timer()?.performance_frequency())
    }

    fn performance_counter(&self) -> Result<u64> {
        Ok(self.context.timer()?.performance_counter())
    }
}

impl DebugGraphics {
    const NES_WIDTH: u32 = 256;
    const NES_HEIGHT: u32 = 240;
    const TITLE: &'static str = "nes-emu";

    const CHAR_WIDTH: i32 = 10;
    const CHAR_HEIGHT: i32 = 12;
    const SPRITES_PER_ROW: i32 = 16;
    const SPRITES_PER_COL: i32 = 16;

    pub fn new(iscale: u32) -> Self {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let window = video_subsystem
            .window(
                Self::TITLE,
                Self::NES_WIDTH * iscale + 512,
                Self::NES_HEIGHT * iscale,
            )
            .position_centered()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().build().unwrap();
        canvas.clear();
        let event_pump = sdl_context.event_pump().unwrap();

        let font_surface = Surface::load_bmp("font.bmp").expect("Could not load font.bmp");

        let texture_creator = canvas.texture_creator();
        let font_texture = texture_creator
            .create_texture_from_surface(font_surface)
            .unwrap();

        let mut character_rects =
            [Rect::new(0, 0, Self::CHAR_WIDTH as u32, Self::CHAR_HEIGHT as u32); 256];
        let mut i = 0;
        for r in 0..Self::SPRITES_PER_ROW {
            for c in 0..Self::SPRITES_PER_COL {
                character_rects[i].x = c * Self::CHAR_WIDTH;
                character_rects[i].y = r * Self::CHAR_HEIGHT;
                i += 1;
            }
        }

        Self {
            context: sdl_context,
            canvas,
            event_pump,
            font_texture,
            character_rects,
            curr_palette: 0,
            show_nametable_boundaries: false,
            iscale,
        }
    }

    fn width(&self) -> u32 {
        Self::NES_WIDTH * self.iscale + 512 
    }

    fn height(&self) -> u32 {
        Self::NES_HEIGHT * self.iscale
    }

    fn draw_pattern_table(
        &mut self,
        pixels: &[[Color; 128]; 128],
        scale: u32,
        x: i32,
        y: i32,
    ) -> Result<()> {
        for r in 0..pixels.len() {
            for c in 0..pixels[r].len() {
                self.canvas.set_draw_color(pixels[r][c]);
                self.canvas.fill_rect(Rect::new(
                    x + (c * scale as usize) as i32,
                    y + (r * scale as usize) as i32,
                    scale,
                    scale,
                ))?;
            }
        }
        Ok(())
    }

    fn draw_palettes(&mut self, palettes: &Vec<Vec<Color>>) -> Result<()> {
        const COLOR_HEIGHT: u32 = 8;
        const COLOR_WIDTH: u32 = 16;
        const BORDER_SCALE: u32 = 2;
        let palette_height = COLOR_HEIGHT + 2 * BORDER_SCALE;
        let palettes_width = COLOR_WIDTH * 4 + 2 * BORDER_SCALE;
        let palettes_y = self.height() - (palette_height * 2) - 256;
        let palettes_x = self.width() - (4 * palettes_width);

        for r in 0..2u32 {
            for c in 0..4u32 {
                let palette_num = (r * 4) + c;
                let x = (palettes_x + (c * palettes_width)) as i32;
                let y = (palettes_y + (r * palette_height)) as i32;
                if self.curr_palette as u32 == palette_num {
                    self.canvas.set_draw_color(Color::WHITE);
                    self.canvas
                        .draw_rect(Rect::new(x, y, palettes_width, palette_height))?;
                }
                for color in 0..4 {
                    self.canvas
                        .set_draw_color(palettes[palette_num as usize][color]);
                    // self.canvas.set_draw_color(Color::WHITE);
                    self.canvas.fill_rect(Rect::new(
                        x + BORDER_SCALE as i32 + (color as i32* COLOR_WIDTH as i32),
                        y + BORDER_SCALE as i32,
                        COLOR_WIDTH,
                        COLOR_HEIGHT,
                    ))?;
                }
            }
        }
        Ok(())
    }

    fn write_text(
        &mut self,
        text: &str,
        x: i32,
        y: i32,
        scale: Option<f32>,
        color: Color,
    ) -> Result<()> {
        let mut curr_x = x;
        let mut curr_y = y;
        let s = scale.unwrap_or(self.iscale as f32);

        self.font_texture.set_color_mod(color.r, color.g, color.b);

        for c in text.as_bytes() {
            if !(*c == b'\n' || *c == b' ') {
                self.canvas.copy(
                    &self.font_texture,
                    Some(self.character_rects[*c as usize]),
                    Some(Rect::new(
                        curr_x,
                        curr_y,
                        (Self::CHAR_WIDTH as f32 * s).round() as u32,
                        (Self::CHAR_HEIGHT as f32 * s).round() as u32,
                    )),
                )?;
            }

            if *c == b'\n' {
                curr_y += (Self::CHAR_HEIGHT + 1) * s as i32;
                curr_x = x;
            } else {
                curr_x += (Self::CHAR_WIDTH + 1) * s as i32;
            }
        }
        Ok(())
    }
}
