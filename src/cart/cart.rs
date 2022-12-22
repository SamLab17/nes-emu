use std::fmt::Debug;

use crate::mem::device::MemoryDevice;
use crate::error::Result;

pub trait Cart : MemoryDevice {
    fn ppu_read(&self, addr: u16, vram: &[u8]) -> Result<u8>;
    fn ppu_write(&mut self, addr: u16, byte: u8, vram: &mut [u8]) -> Result<()>; 
}

impl Debug for dyn Cart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Cart: {}", self.name())
    }
}

pub type Cartridge = Box<dyn Cart + 'static>;