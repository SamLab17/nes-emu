use crate::cpu::cpu::Cpu;
use crate::error::Result;
use nes_emu::ppu::ppu::Frame;
use sdl2::event::Event;

pub trait NesGraphics {
    fn render_frame(&mut self, frame: Frame, cpu: &mut Cpu) -> Result<()>;
    fn events(&mut self) -> Vec<Event>;

    fn performance_frequency(&self) -> Result<u64>;
    fn performance_counter(&self) -> Result<u64>;
}
