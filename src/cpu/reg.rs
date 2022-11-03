use bitflags::bitflags;
use std::fmt;

bitflags! {
    pub struct StatusFlags: u8 {
        const CARRY = 0b00000001;
        const ZERO = 0b00000010;
        const INTERRUPT_DISABLE = 0b00000100;
        const DECIMAL = 0b0000100;
        const BREAK = 0b0001000;
        const UNUSED = 0b00100000;
        const OVERFLOW = 0b01000000;
        const NEGATIVE = 0b10000000;
    }
}

impl Default for StatusFlags {
    fn default() -> Self {
        StatusFlags { bits: 0b00100000 }
    }
}

#[derive(Default, Clone, Debug)]
pub struct Registers {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub pc: u16,
    pub sp: u8,
    pub status: StatusFlags,
}

impl fmt::Display for Registers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "A      : 0x{:X}", self.a)?;
        write!(f, "X      : 0x{:X}", self.x)?;
        write!(f, "Y      : 0x{:X}", self.y)?;
        write!(f, "PC     : 0x{:X}", self.pc)?;
        write!(f, "SP     : 0x{:X}", self.sp)?;
        write!(f, "Status : 0x{:X}", self.status)
    }
}
