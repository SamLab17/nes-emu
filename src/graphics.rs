use nes_emu::ppu::ppu::Frame;
use sdl2::EventPump;
use sdl2::pixels::Color;
use sdl2::event::{Event, EventPollIterator};
use sdl2::keyboard::Keycode;
use sdl2::rect::{Point, Rect};
use sdl2::render::Canvas;
use sdl2::video::Window;
use std::time::Duration;
use crate::error::Result;

pub struct NesEmuGraphics {
    canvas: Canvas<Window>,
    event_pump: EventPump,
    iscale: u32
}

impl NesEmuGraphics {
    const WIDTH: u32 = 256;
    const HEIGHT: u32 = 240;
    const DEFAULT_SCALE: u32 = 2;
    const TITLE: &'static str = "nes-emu";

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
        
        NesEmuGraphics { canvas, event_pump, iscale }
    }

    pub fn events(&mut self) -> EventPollIterator {
        self.event_pump.poll_iter()
    }

    pub fn render_frame(&mut self, frame: Frame) -> Result<()> {
        for r in 0..frame.len() {
            for c in 0..frame[r].len() {
                self.canvas.set_draw_color(frame[r][c]);
                self.canvas.fill_rect(Rect::new(
                    (c * (self.iscale as usize)).try_into().unwrap(),
                    (r * (self.iscale as usize)).try_into().unwrap(),
                    self.iscale,
                    self.iscale
                ))?;
            }
        }
        self.canvas.present();
        Ok(())
    }

}