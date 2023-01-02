use std::error::Error;
use std::fmt::{Debug};
use std::fmt;

use crate::error::Result;
use crate::ines::parse::MirrorType;

pub enum PpuMemoryError {
    PpuReadOnly(u16),
    PpuInvalidAddress(u16)
}

impl Error for PpuMemoryError {}

impl fmt::Display for PpuMemoryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PpuMemoryError::PpuReadOnly(a) => write!(f, "PpuReadOnly(0x{:X})", *a),
            PpuMemoryError::PpuInvalidAddress(a) => write!(f, "PpuInvalidAddress(0x{:X})", *a),
        }
    }
}

impl fmt::Debug for PpuMemoryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

pub fn ppu_rd_only(addr: u16) -> Box<dyn Error> {
    Box::new(PpuMemoryError::PpuReadOnly(addr))
}

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

impl Debug for dyn Cart + Send {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Cart: {}", self.name())
    }
}

pub type Cartridge = Box<dyn Cart + 'static + Send>;

/*
Nametable numbers:
          |
     00   |   01
  -----------------
     10  |   11
         |

Nametable Address:
NNOO OOOO OOOO
N = nametable number
O = offset
*/
pub fn nametable_addr(mut addr: u16, mirror_type: MirrorType) -> u16 {
    addr &= 0xFFF;
    match mirror_type {
        MirrorType::Vertical => match addr {
            0x000..=0x3FF | 0x800..=0xBFF => addr & 0x3FF,
            0x400..=0x7FF | 0xC00..=0xFFF => (addr & 0x3FF) | 0x400,
            _ => panic!("impossible"),
        },
        MirrorType::Horizontal => match addr {
            0x000..=0x7FF => addr & 0x3FF,
            0x800..=0xFFF => (addr & 0x3FF) | 0x400,
            _ => panic!("impossible"),
        },
        MirrorType::OneScreenLow => match addr {
            0x000..=0xFFF => addr & 0x3FF,
            _ => panic!("impossible"),
        },
        MirrorType::OneScreenHigh => match addr {
            0x000..=0xFFF => (addr & 0x3FF) | 0x400,
            _ => panic!("impossible"),
        },
    }
}

#[cfg(test)]
mod cart_tests {
    use crate::{cart::cart::nametable_addr, ines::parse::MirrorType};

    #[test]
    fn test_mirroring() {
        for off in 0..0x400 {
            assert_eq!(
                nametable_addr(0x2400 + off, MirrorType::Horizontal),
                0x000 + off
            );
            assert_eq!(
                nametable_addr(0x2000 + off, MirrorType::Horizontal),
                0x000 + off
            );
            assert_eq!(
                nametable_addr(0x2C00 + off, MirrorType::Horizontal),
                0x400 + off
            );
            assert_eq!(
                nametable_addr(0x2800 + off, MirrorType::Horizontal),
                0x400 + off
            );

            assert_eq!(
                nametable_addr(0x2800 + off, MirrorType::Vertical),
                0x000 + off
            );
            assert_eq!(
                nametable_addr(0x2000 + off, MirrorType::Vertical),
                0x000 + off
            );
            assert_eq!(
                nametable_addr(0x2C00 + off, MirrorType::Vertical),
                0x400 + off
            );
            assert_eq!(
                nametable_addr(0x4C00 + off, MirrorType::Vertical),
                0x400 + off
            );
        }
    }
}