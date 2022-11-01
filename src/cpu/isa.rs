use crate::{error::NesEmuError, cpu::{reg::Registers, cpu::Cpu}, mem::device::{MemoryDevice, MemoryError}};
use std::fmt;

#[derive(Clone, Copy, Debug)]
pub enum AddressingMode {
    Accumulator,
    Absolute(u16),
    AbsoluteX(u16),
    AbsoluteY(u16),
    Immediate(u8),
    Implied,
    Indirect(u16),
    XIndirect(u8),
    IndrectY(u8),
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

#[derive(Clone, Copy, Debug)]
pub struct Instr {
    op: Opcode,
    mode: AddressingMode,
    num_cycles: u8
}

#[derive(Debug)]
struct RuntimeError {
    err: Box<dyn NesEmuError>,
    reg_state: Registers
}

impl NesEmuError for RuntimeError {}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error occurred: {}\nRegister State:\n{}", self.err, self.reg_state)
    }
}

fn run_instr(i: Instr, cpu: &mut Cpu, mem: &mut dyn MemoryDevice) -> Result<(), RuntimeError> {
    Err(RuntimeError { err: (Box::new(MemoryError::InvalidAddress(0))), reg_state: cpu.reg.clone() })
}

fn fetch_instr(cpu: &mut Cpu) -> Option<Instr> {
    
}