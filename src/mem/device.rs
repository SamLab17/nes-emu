use crate::error::Result;
use std::{error::Error, fmt};
use std::fmt::Debug;

#[derive(Clone, Copy)]
pub enum MemoryError {
    ReadOnly(u16),
    WriteOnly(u16),
    InvalidAddress(u16),
}

pub fn inv_addr(addr: u16) -> Box<MemoryError> {
    Box::new(MemoryError::InvalidAddress(addr))
}

pub fn rd_only(addr: u16) -> Box<MemoryError> {
    Box::new(MemoryError::ReadOnly(addr))
}

pub fn wr_only(addr: u16) -> Box<MemoryError> {
    Box::new(MemoryError::WriteOnly(addr))
}

impl Error for MemoryError {}

impl fmt::Display for MemoryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MemoryError::ReadOnly(a) => write!(f, "ReadOnly(0x{:X})", *a),
            MemoryError::WriteOnly(a) => write!(f, "WriteOnly(0x{:X})", *a),
            MemoryError::InvalidAddress(a) => write!(f, "InvalidAddress(0x{:X})", *a),
        }
    }
}

impl fmt::Debug for MemoryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

pub trait MemoryDevice {
    fn name(&self) -> String;

    // A shame that this "self" ref has to be mut, but the PPU can change its state
    // based on reads
    fn read(&mut self, addr: u16) -> Result<u8>;
    fn write(&mut self, addr: u16, byte: u8) -> Result<()>;

    fn write_many(&mut self, start_addr: u16, bytes: &[u8]) -> Result<()> {
        let mut addr = start_addr;
        for byte in bytes.iter() {
            self.write(addr, *byte)?;
            addr += 1
        }
        Ok(())
    }
}

impl Debug for dyn MemoryDevice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}
