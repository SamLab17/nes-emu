use std::fmt;
use std::{ops::{Add, Index}, collections::HashMap, error::Error};

use lazy_static::lazy_static;
use strum::IntoEnumIterator;

use crate::error::Result;

use super::{isa::{Instr, AddressingMode, Opcode}, reg::StatusFlags, deref::effective_addr};
use crate::{cpu::cpu::Cpu};
use crate::mem::utils::{make_address, page_num};
use super::utils::is_negative;

use super::deref::{deref_byte, deref_address};

#[derive(Debug, Clone)]
enum ExecutionError {
    InvalidInstruction(Instr)
}

impl Error for ExecutionError {}

impl fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ExecutionError::InvalidInstruction(i) => write!(f, "Attempted to run invalid instruction: {:?}", i)
        }
    }
}

// Function which implements the behavior of an instruction
// returns the number of _extra_ cycles needed to run the instruction
// (this extra will be added to number of cycles in instruction matrix)
type OpcodeFn = fn(AddressingMode, &mut Cpu) -> Result<u8>;

lazy_static! {
    static ref OPCODE_FUNCTIONS: HashMap<Opcode, OpcodeFn> = {
        fn lookup_opcode_fn(op: Opcode) -> OpcodeFn {
            use Opcode::*;
            match op {
                LDA => lda,

                _ => panic!("NYI")
            }
        }

        let mut m = HashMap::new();
        for opcode in Opcode::iter() {
            m.insert(opcode, lookup_opcode_fn(opcode));
        }
        m
    };
}

fn exec_instr(i: Instr, cpu: &mut Cpu) -> Result<u8> {
    // Lookup opcode function
    if let Some(instr_fn) = OPCODE_FUNCTIONS.get(&i.op).map(|f: &OpcodeFn| *f) {
        let extra_cycles = instr_fn(i.mode, cpu)?;
        Ok(i.num_cycles + extra_cycles)
    } else {
        Err(Box::new(ExecutionError::InvalidInstruction(i)))
    }
    // Run instruction
}

// Returns whether this addressing mode will cross a page boundary
// (which typically incurs an extra cycle cost)
fn cross_page_boundary(am: AddressingMode, cpu: &Cpu) -> bool {
    use AddressingMode::*;
    match am {
        AbsoluteX(addr) => page_num(addr) != page_num(addr + (cpu.reg.x as u16)),
        AbsoluteY(addr) => page_num(addr) != page_num(addr + (cpu.reg.y as u16)),
        IndirectY(offset) => {
            let lo = cpu.bus.read(offset as u16).unwrap();
            let hi = cpu.bus.read((offset + 1) as u16).unwrap();
            let effective = make_address(lo, hi); 

            page_num(effective) != page_num(effective + (cpu.reg.y as u16))
        }
        Relative(_) => {
            page_num(cpu.reg.pc) != page_num(deref_address(am, cpu).unwrap())
        }
        // Other addressing modes don't cross page boundaries
        _ => false
    }
}

// Conditionally set the "Negative" status flag based on a value
fn set_negative_flag(cpu: &mut Cpu, val: u8) {
    cpu.reg.status.set(StatusFlags::NEGATIVE, is_negative(val));
}

// Conditionally set the "Zero" status flag based on a value
fn set_zero_flag(cpu: &mut Cpu, val: u8) {
    cpu.reg.status.set(StatusFlags::ZERO, val == 0);
}

/*
 *       Instruction Implementations
 */

fn lda(am: AddressingMode, cpu: &mut Cpu) -> Result<u8> {
    cpu.reg.a = deref_byte(am, cpu)?;
    // Check if we cross a boundary before updating any state!
    let crossed = cross_page_boundary(am, cpu);

    set_negative_flag(cpu, cpu.reg.a);
    set_zero_flag(cpu, cpu.reg.a);

    if crossed {
        Ok(1)
    } else {
        Ok(0)
    }
}

fn jmp(am: AddressingMode, cpu: &mut Cpu) -> Result<u8> {
    cpu.reg.pc = deref_address(am, cpu)?;
    Ok(0)
}

fn inc(am: AddressingMode, cpu: &mut Cpu) -> Result<u8> {
    let addr = effective_addr(am, cpu)?;
    let val = cpu.bus.read(addr)? + 1;
    cpu.bus.write(addr, val)?;

    set_negative_flag(cpu, val);
    set_zero_flag(cpu, val);
    Ok(0)
}

fn inx(am: AddressingMode, cpu: &mut Cpu) -> Result<u8> {
    cpu.reg.x += 1;
    set_negative_flag(cpu, cpu.reg.x);
    set_zero_flag(cpu, cpu.reg.x);
    Ok(0)
}

#[cfg(test)]
mod exec_tests {
    use crate::cpu::cpu::Cpu;
    use super::AddressingMode::*;

    use super::cross_page_boundary;

    #[test]
    fn cross_page_boundary_test() {
        let mut cpu = Cpu::mock(&[0; 0x1FFF]);
        cpu.reg.x = 0xFF;
        cpu.reg.y = 0xFF;
        assert!(cross_page_boundary(AbsoluteX(0x17), &cpu));
        assert!(cross_page_boundary(AbsoluteX(0x01), &cpu));
        assert!(cross_page_boundary(AbsoluteX(0xFF), &cpu));
        assert!(cross_page_boundary(AbsoluteY(0x17), &cpu));
        assert!(cross_page_boundary(AbsoluteY(0x01), &cpu));
        assert!(cross_page_boundary(AbsoluteY(0xFF), &cpu));

        cpu.reg.x = 0x20;
        cpu.reg.y = 0x20;
        assert!(!cross_page_boundary(AbsoluteX(0x17), &cpu));
        assert!(cross_page_boundary(AbsoluteX(0xEE), &cpu));
        assert!(!cross_page_boundary(AbsoluteX(0x00), &cpu));
        assert!(!cross_page_boundary(AbsoluteY(0x17), &cpu));
        assert!(cross_page_boundary(AbsoluteY(0xEE), &cpu));
        assert!(!cross_page_boundary(AbsoluteY(0x00), &cpu));

        cpu.reg.y = 0x80;
        cpu.bus.write(0x42, 0xFF).unwrap();
        cpu.bus.write(0x43, 0x00).unwrap();
        assert!(cross_page_boundary(IndirectY(0x42), &cpu));

        cpu.bus.write(0x42, 0x00).unwrap();
        cpu.bus.write(0x43, 0x10).unwrap();
        assert!(!cross_page_boundary(IndirectY(0x42), &cpu));


        cpu.reg.pc = 0x01FF;
        // -1 jump
        assert!(!cross_page_boundary(Relative(0xFF), &cpu));
        // -64 jump
        assert!(!cross_page_boundary(Relative(0xC0), &cpu));
        // + 1 jump
        assert!(cross_page_boundary(Relative(0x1), &cpu));
        // + 64 jump
        assert!(cross_page_boundary(Relative(0x40), &cpu));

        cpu.reg.pc = 0x0100;
        // -1 jump
        assert!(cross_page_boundary(Relative(0xFF), &cpu));
        // -64 jump
        assert!(cross_page_boundary(Relative(0xC0), &cpu));
        // + 1 jump
        assert!(!cross_page_boundary(Relative(0x1), &cpu));
        // + 64 jump
        assert!(!cross_page_boundary(Relative(0x40), &cpu));
    }
}
