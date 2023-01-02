use crate::{error::Result, ppu::ppu::Frame};
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::VideoSubsystem;

use super::graphics::{CpuInfo, NesGraphics};

pub struct SimpleGraphics {
    canvas: Canvas<Window>,
    iscale: u32,
}

impl NesGraphics for SimpleGraphics {
    fn render_frame(&mut self, frame: Frame, _info: CpuInfo) -> Result<()> {
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

        self.canvas.present();
        Ok(())
    }

    fn process_events(&mut self, _events: &Vec<sdl2::event::Event>) {}

}

impl SimpleGraphics {
    const WIDTH: u32 = 256;
    const HEIGHT: u32 = 240;
    const TITLE: &'static str = "nes-emu";

    pub fn new(iscale: u32, video: VideoSubsystem) -> Self {
        let canvas = video
            .window(Self::TITLE, Self::WIDTH * iscale, Self::HEIGHT * iscale)
            .position_centered()
            .build()
            .unwrap()
            .into_canvas()
            .build()
            .unwrap();

        Self { canvas, iscale }
    }
}
