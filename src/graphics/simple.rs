use crate::cpu::cpu::Cpu;
use crate::error::Result;
use nes_emu::ppu::ppu::Frame;
use sdl2::event::Event;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::{EventPump, Sdl};

use super::graphics::NesGraphics;

pub struct SimpleGraphics {
    context: Sdl,
    canvas: Canvas<Window>,
    event_pump: EventPump,
    iscale: u32,
}

impl NesGraphics for SimpleGraphics {
    fn render_frame(&mut self, frame: Frame, _cpu: &mut Cpu) -> Result<()> {
        let frame = frame.borrow();
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

    fn events(&mut self) -> Vec<Event> {
        self.event_pump.poll_iter().collect()
    }

    fn performance_frequency(&self) -> Result<u64> {
        Ok(self.context.timer()?.performance_frequency())
    }

    fn performance_counter(&self) -> Result<u64> {
        Ok(self.context.timer()?.performance_counter())
    }
}

impl SimpleGraphics {
    const WIDTH: u32 = 256;
    const HEIGHT: u32 = 240;
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

        Self {
            context: sdl_context,
            canvas,
            event_pump,
            iscale,
        }
    }
}
