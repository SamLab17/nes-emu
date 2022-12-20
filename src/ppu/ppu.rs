use crate::mem::device::{MemoryDevice, MemoryError};
use crate::error::Result;

pub struct Ppu {

}

impl Ppu {
}

impl Default for Ppu {
    fn default() -> Self {
        Self {  }
    }
}

impl MemoryDevice for Ppu {

    fn name(&self) -> String { "PPU".into() }

    fn read(&self, addr: u16) -> Result<u8> {
        if addr < 0x2000 || addr > 0x3FFF {
            Err(Box::new(MemoryError::InvalidAddress(addr)))
        } else {
            todo!("PPU read registers")
        }
    }

    fn write(&mut self, addr: u16, byte: u8) -> Result<()> {
        todo!("PPU write registers")
    }
}