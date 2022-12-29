use crate::mem::error::MemoryError;

use super::cart::{Cart, Cartridge};

pub struct MockCartridge {}

impl Cart for MockCartridge {

    fn name(&self) -> String {
        "Mock Cartridge".into()
    }

    fn read(&mut self, addr: u16) -> crate::error::Result<u8> {
        Err(Box::new(MemoryError::InvalidAddress(addr)))
    }

    fn write(&mut self, addr: u16, _byte: u8) -> crate::error::Result<()> {
        Err(Box::new(MemoryError::InvalidAddress(addr)))
    }
    fn ppu_read(&self, addr: u16, _vram: &[u8]) -> crate::error::Result<u8> {
        Err(Box::new(MemoryError::InvalidAddress(addr)))
    }

    fn ppu_write(&mut self, addr: u16, _byte: u8, _vram: &mut [u8]) -> crate::error::Result<()> {
        Err(Box::new(MemoryError::InvalidAddress(addr)))
    }
}

pub fn mock_cart() -> Cartridge {
    Box::new(MockCartridge {})
}
