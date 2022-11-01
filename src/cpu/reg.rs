use bitflags::bitflags;
use std::fmt;

bitflags! {
    pub struct StatusFlags: u8 {
        const Carry = 0b00000001;
        const Zero = 0b00000010;
        const InterruptDisable = 0b00000100;
        const Decimal = 0b0001000;
        const Overflow = 0b01000000;
        const Negative = 0b10000000;
    }
}

impl Default for StatusFlags {
    fn default() -> Self {
        StatusFlags { bits: 0 }
    }
}

#[derive(Default, Clone, Debug)]
pub struct Registers {
    a: u8,
    x: u8,
    y: u8,
    pc: u16,
    sp: u8,
    flags: StatusFlags
}

impl fmt::Display for Registers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "A     : 0x{:X}", self.a)?;
        write!(f, "X     : 0x{:X}", self.x)?;
        write!(f, "Y     : 0x{:X}", self.y)?;
        write!(f, "PC    : 0x{:X}", self.pc)?;
        write!(f, "SP    : 0x{:X}", self.sp)?;
        write!(f, "Flags : 0x{:X}", self.flags)
    }
}
