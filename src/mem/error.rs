use std::{error::Error, fmt};

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
