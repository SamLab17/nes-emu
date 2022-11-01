use crate::{cpu::{reg::Registers, cpu::Cpu}, mem::device::{MemoryDevice, MemoryError}};
use std::fmt;

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
    ZeroPageY(u8)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
    pub num_cycles: u8
}

// #[derive(Debug)]
// struct RuntimeError {
//     err: Box<dyn NesEmuError>,
//     reg_state: Registers
// }

// impl NesEmuError for RuntimeError {}

// impl fmt::Display for RuntimeError {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "Error occurred: {}\nRegister State:\n{}", self.err, self.reg_state)
//     }
// }
