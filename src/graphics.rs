use crate::error::Result;
use nes_emu::ppu::ppu::Frame;
use sdl2::event::EventPollIterator;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture};
use sdl2::surface::Surface;
use sdl2::video::Window;
use sdl2::{EventPump, Sdl};

pub struct NesEmuGraphics {
    context: Sdl,
    canvas: Canvas<Window>,
    event_pump: EventPump,
    font_texture: Texture,
    character_rects: [Rect; 256],
    iscale: u32,
}

impl NesEmuGraphics {
    const WIDTH: u32 = 256;
    const HEIGHT: u32 = 240;
    const TITLE: &'static str = "nes-emu";

    const CHAR_WIDTH: i32 = 10;
    const CHAR_HEIGHT: i32 = 12;
    const SPRITES_PER_ROW: i32 = 16;
    const SPRITES_PER_COL: i32 = 16;

    pub fn new(iscale: u32) -> Self {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let window = video_subsystem
            .window(Self::TITLE, Self::WIDTH * iscale, Self::HEIGHT * iscale)
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

        NesEmuGraphics {
            context: sdl_context,
            canvas,
            event_pump,
            font_texture,
            character_rects,
            iscale,
        }
    }

    pub fn events(&mut self) -> EventPollIterator {
        self.event_pump.poll_iter()
    }

    fn write_text(&mut self, text: &str, x: i32, y: i32, scale: Option<i32>, color: Color) -> Result<()> {
        let mut curr_x = x;
        let mut curr_y = y;
        let s = scale.unwrap_or(self.iscale as i32);

        self.font_texture.set_color_mod(color.r, color.g, color.b);

        for c in text.as_bytes() {
            if !(*c == b'\n' || *c == b' ') {
                self.canvas.copy(
                    &self.font_texture,
                    Some(self.character_rects[*c as usize]),
                    Some(Rect::new(
                        curr_x,
                        curr_y,
                        Self::CHAR_WIDTH as u32 * self.iscale,
                        Self::CHAR_HEIGHT as u32 * self.iscale,
                    )),
                )?;
            }

            if *c == b'\n' {
                curr_y += (Self::CHAR_HEIGHT+2) * s;
                curr_x = x;
            } else {
                curr_x += (Self::CHAR_WIDTH+2) * s;
            }
        }
        Ok(())
    }


    pub fn render_frame(&mut self, frame: Frame) -> Result<()> {
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

        self.write_text("Nintendo Entertainment\nSystem", 10, 10, Some(2), Color::RED)?;

        self.canvas.present();
        Ok(())
    }

    pub fn performance_frequency(&self) -> Result<u64> {
        Ok(self.context.timer()?.performance_frequency())
    }

    pub fn performance_counter(&self) -> Result<u64> {
        Ok(self.context.timer()?.performance_counter())
    }
}
