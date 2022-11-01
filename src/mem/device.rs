use std::fmt;
use crate::error::NesEmuError;

#[derive(Debug, Clone, Copy)]
pub enum MemoryError {
    ReadOnly(u16),
    InvalidAddress(u16),
}

impl NesEmuError for MemoryError {}

impl fmt::Display for MemoryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MemoryError::ReadOnly(addr) => write!(f, "Address 0x{:x} is read-only.", addr),
            MemoryError::InvalidAddress(addr) => write!(f, "Address 0x{:x} is invalid", addr)
        }
    }
}

pub trait MemoryDevice {
    fn read(&self, addr: u16) -> Option<u8>;
    fn write(&mut self, addr: u16, byte: u8) -> Result<(), MemoryError>;

    fn write_many(&mut self, start_addr: u16, bytes: &[u8]) -> Result<(), MemoryError> {
        let mut addr = start_addr;
        for byte in bytes.iter() {
            self.write(addr, *byte)?;
            addr += 1
        }
        Ok(())
    }
}