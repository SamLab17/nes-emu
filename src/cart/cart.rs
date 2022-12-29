use std::error::Error;
use std::fmt::{Debug};
use std::fmt;

use crate::error::Result;

pub enum PpuMemoryError {
    // PpuReadOnly(u16),
    // PpuWriteOnly(u16),
    PpuInvalidAddress(u16)
}

impl Error for PpuMemoryError {}

impl fmt::Display for PpuMemoryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            // PpuMemoryError::PpuReadOnly(a) => write!(f, "PpuReadOnly(0x{:X})", *a),
            // PpuMemoryError::PpuWriteOnly(a) => write!(f, "PpuWriteOnly(0x{:X})", *a),
            PpuMemoryError::PpuInvalidAddress(a) => write!(f, "PpuInvalidAddress(0x{:X})", *a),
        }
    }
}

impl fmt::Debug for PpuMemoryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

// pub fn ppu_rd_only(addr: u16) -> Box<dyn Error> {
//     Box::new(PpuMemoryError::PpuReadOnly(addr))
// }

// pub fn ppu_wr_only(addr: u16) -> Box<dyn Error> {
//     Box::new(PpuMemoryError::PpuWriteOnly(addr))
// }

pub fn ppu_inv_addr(addr: u16) -> Box<dyn Error> {
    Box::new(PpuMemoryError::PpuInvalidAddress(addr))
}

pub trait Cart {
    fn name(&self) -> String;
    fn read(&mut self, addr: u16) -> Result<u8>;
    fn write(&mut self, addr: u16, byte: u8) -> Result<()>;
    fn ppu_read(&self, addr: u16, vram: &[u8]) -> Result<u8>;
    fn ppu_write(&mut self, addr: u16, byte: u8, vram: &mut [u8]) -> Result<()>; 
}

impl Debug for dyn Cart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Cart: {}", self.name())
    }
}

pub type Cartridge = Box<dyn Cart + 'static>;