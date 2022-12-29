use crate::cpu::cpu::Cpu;
use crate::error::Result;
use nes_emu::ppu::ppu::Frame;
use sdl2::event::Event;

use super::debug::DebugGraphics;
use super::simple::SimpleGraphics;

pub trait NesGraphics {
    fn render_frame(&mut self, frame: Frame, cpu: &mut Cpu) -> Result<()>;
    fn events(&mut self) -> Vec<Event>;

    fn performance_frequency(&self) -> Result<u64>;
    fn performance_counter(&self) -> Result<u64>;
}


pub struct GraphicsBuilder {
    iscale: u32,
    debug: bool
}

impl GraphicsBuilder {
    pub fn new() -> Self {
        GraphicsBuilder { iscale: 3, debug: false }
    }
    pub fn debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }
    pub fn scale(mut self, s: Option<u32>) -> Self {
        if let Some(s) = s {
            self.iscale = s;
        }
        self
    }
    pub fn build(self) -> Box<dyn NesGraphics> {
        match self.debug {
            true => Box::new(DebugGraphics::new(self.iscale)),
            false => Box::new(SimpleGraphics::new(self.iscale))
        }
    }
}