use crate::{error::NesEmuError, cpu::{reg::Registers, cpu::Cpu}, mem::device::{MemoryDevice, MemoryError}};
use std::fmt;

#[derive(Clone, Copy)]
enum AddressingMode {
    Acc,
    Abs,
    AbsX,
    AbsY,
    Imm,
    Imp,
    Ind,
    XInd,
    IndY,
    Rel,
    Zpg,
    ZpgX,
    ZpgY
}

#[derive(Clone, Copy)]
enum Opcode {
    BRK,
}

#[derive(Clone, Copy)]
struct Instr {
    op: Opcode,
    mode: AddressingMode
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