use crate::error::Result;
use std::{error::Error, fmt};
use std::fmt::Debug;

#[derive(Debug, Clone, Copy)]
pub enum MemoryError {
    ReadOnly(u16),
    InvalidAddress(u16),
}

impl Error for MemoryError {
    fn description(&self) -> &str {
        match self {
            MemoryError::ReadOnly(_) => "Address is read-only",
            MemoryError::InvalidAddress(_) => "Address is invalid",
        }
    }
}

impl fmt::Display for MemoryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

pub trait MemoryDevice {
    fn name(&self) -> String;
    fn read(&self, addr: u16) -> Result<u8>;
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