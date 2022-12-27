use std::fmt;

use strum_macros::EnumIter;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AddressingMode {
    Accumulator,
    Absolute(u16),
    AbsoluteX(u16),
    AbsoluteY(u16),
    Immediate(u8),
    Implied,
    Indirect(u16),
    XIndirect(u8),
    IndirectY(u8),
    Relative(u8),
    ZeroPage(u8),
    ZeroPageX(u8),
    ZeroPageY(u8),
}

impl fmt::Display for AddressingMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use AddressingMode::*;
        match *self {
            Accumulator => write!(f, "A"),
            Absolute(addr) => write!(f, "${:04X} (Abs)", addr),
            AbsoluteX(addr) => write!(f, "${:04X},X (AbsX)", addr),
            AbsoluteY(addr) => write!(f, "${:04X},Y (AbsY)", addr),
            Immediate(val) => write!(f, "#${:02X} (Imm)", val),
            Implied => write!(f, " (Imp)"),
            Indirect(addr) => write!(f, "(${:04X}) (Ind)", addr),
            XIndirect(offset) => write!(f, "(${:02X}, X) (XInd)", offset),
            IndirectY(offset) => write!(f, "(${:02X}), Y (IndY)", offset),
            Relative(offset) => write!(f, "${:02X} (Rel)", offset),
            ZeroPage(offset) => write!(f, "${:02X} (Zpg)", offset),
            ZeroPageX(offset) => write!(f, "${:02X},X (ZpgX)", offset),
            ZeroPageY(offset) => write!(f, "${:02X},Y (ZpgY)", offset),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, EnumIter)]
pub enum Opcode {
    ADC,
    AND,
    ASL,
    BCC,
    BCS,
    BEQ,
    BIT,
    BMI,
    BNE,
    BPL,
    BRK,
    BVC,
    BVS,
    CLC,
    CLD,
    CLI,
    CLV,
    CMP,
    CPX,
    CPY,
    DEC,
    DEX,
    DEY,
    EOR,
    INC,
    INX,
    INY,
    JMP,
    JSR,
    LAX,
    LDA,
    LDX,
    LDY,
    LSR,
    NOP,
    ORA,
    PHA,
    PHP,
    PLA,
    PLP,
    ROL,
    ROR,
    RTI,
    RTS,
    SBC,
    SEC,
    SED,
    SEI,
    STA,
    STX,
    STY,
    TAX,
    TAY,
    TSX,
    TXA,
    TXS,
    TYA,
    INVALID,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Instr {
    pub op: Opcode,
    pub mode: AddressingMode,
}

impl fmt::Display for Instr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if matches!(self.mode, AddressingMode::Implied) {
            // to prevent trailing space
            write!(f, "{:?}", self.op)
        } else {
            write!(f, "{:?} {}", self.op, self.mode)
        }
    }
}

#[cfg(test)]
mod isa_tests {
    use super::{Instr, Opcode, AddressingMode};
    use Opcode::*;
    use AddressingMode::*;

    #[test]
    fn instr_display_test() {
        let i1 = Instr{op: ADC, mode: Absolute(0xCAFE)};
        assert_eq!(format!("{}", i1), "ADC $CAFE (Abs)".to_string());

        let i2 = Instr{op: JMP, mode: Indirect(0x1234)};
        assert_eq!(format!("{}", i2), "JMP ($1234) (Ind)".to_string());

        let i3 = Instr{op: ASL, mode: Accumulator};
        assert_eq!(format!("{}", i3), "ASL A".to_string());

        let i4 = Instr{op: BRK, mode: Implied};
        assert_eq!(format!("{}", i4), "BRK".to_string());
    }
}
