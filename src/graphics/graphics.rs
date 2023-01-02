use crate::cpu::isa::Instr;
use crate::cpu::reg::Registers;
use crate::error::Result;
use crate::ppu::ppu::{PatternTable, OamSprite, Frame};
use sdl2::VideoSubsystem;
use sdl2::event::Event;
use sdl2::pixels::Color;

use super::debug::DebugGraphics;
use super::simple::SimpleGraphics;

pub struct CpuInfo {
    pub sprites: Vec<OamSprite>,
    pub palettes: Vec<Vec<Color>>,
    // These pattern tables have to have a hard-wired palette, we could also just store the index
    // and have the graphics window index into the palettes
    pub pattern_tables: (PatternTable, PatternTable),
    pub instructions: Vec<(u16, Instr)>,
    pub registers: Registers
}

pub trait NesGraphics {
    fn render_frame(&mut self, frame: Frame, info: CpuInfo) -> Result<()>;
    fn process_events(&mut self, events: &Vec<Event>);
}


pub struct GraphicsBuilder {
    iscale: u32,
    debug: bool,
    video: VideoSubsystem
}

impl GraphicsBuilder {
    pub fn new(video: VideoSubsystem) -> Self {
        GraphicsBuilder { iscale: 3, debug: false, video }
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
            true => Box::new(DebugGraphics::new(self.iscale, self.video)),
            false => Box::new(SimpleGraphics::new(self.iscale, self.video))
        }
    }
}