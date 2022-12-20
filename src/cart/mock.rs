use crate::mem::device::{MemoryError, MemoryDevice};

pub struct MockCartridge {}

impl MemoryDevice for MockCartridge {
    fn name(&self) -> String { "Mock Cartridge".into() }

    fn read(&self, addr: u16) -> crate::error::Result<u8> {
        Err(Box::new(MemoryError::InvalidAddress(addr)))
    }

    fn write(&mut self, addr: u16, byte: u8) -> crate::error::Result<()> {
        Err(Box::new(MemoryError::InvalidAddress(addr)))
    }
}