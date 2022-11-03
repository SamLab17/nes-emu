use std::fmt;
use std::{collections::HashMap, error::Error};

use lazy_static::lazy_static;
use strum::IntoEnumIterator;

use crate::error::Result;

use super::cpu::{Interrupt, STACK_OFFSET};
use super::decode::instr_lookup::num_cycles_for_instr;
use super::utils::is_negative;
use super::{
    deref::effective_addr,
    isa::{AddressingMode, Instr, Opcode},
    reg::StatusFlags,
};
use crate::cpu::cpu::Cpu;
use crate::mem::utils::{hi_byte, lo_byte, make_address, page_num};

use super::deref::{deref_address, deref_byte};

#[derive(Debug, Clone)]
enum ExecutionError {
    InvalidInstruction(Instr),
}

impl Error for ExecutionError {}

impl fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ExecutionError::InvalidInstruction(i) => {
                write!(f, "Attempted to run invalid instruction: {:?}", i)
            }
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
                ADC => adc,
                AND => and,
                ASL => asl,
                BCC => bcc,
                BCS => bcs,
                BEQ => beq,
                BIT => bit,
                BMI => bmi,
                BNE => bne,
                BPL => bpl,
                BRK => brk,
                BVC => bvc,
                BVS => bvs,
                CLC => clc,
                CLD => cld,
                CLI => cli,
                CLV => clv,
                CMP => cmp,
                CPX => cpx,
                CPY => cpy,
                DEC => dec,
                DEX => dex,
                DEY => dey,
                EOR => eor,
                INC => inc,
                INX => inx,
                INY => iny,
                JMP => jmp,
                JSR => jsr,
                LDA => lda,
                LDX => ldx,
                LDY => ldy,
                LSR => lsr,
                NOP => nop,
                ORA => ora,
                PHA => pha,
                PHP => php,
                PLA => pla,
                PLP => plp,

                _ => panic!("NYI"),
            }
        }

        let mut m = HashMap::new();
        for opcode in Opcode::iter() {
            m.insert(opcode, lookup_opcode_fn(opcode));
        }
        m
    };
}

pub fn exec_instr(i: Instr, cpu: &mut Cpu) -> Result<u8> {
    // Lookup opcode function
    if let Some(instr_fn) = OPCODE_FUNCTIONS.get(&i.op).map(|f: &OpcodeFn| *f) {
        let num_cycles = num_cycles_for_instr(i).ok_or(ExecutionError::InvalidInstruction(i))?;
        let extra_cycles = instr_fn(i.mode, cpu)?;

        Ok(num_cycles + extra_cycles)
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
        Relative(_) => page_num(cpu.reg.pc) != page_num(deref_address(am, cpu).unwrap()),
        // Other addressing modes don't cross page boundaries
        _ => false,
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

fn push_stack(cpu: &mut Cpu, byte: u8) -> Result<()> {
    let sp = (cpu.reg.sp as u16) + STACK_OFFSET;
    cpu.bus.write(sp, byte)?;
    cpu.reg.sp -= 1;
    Ok(())
}

fn push_stack_addr(cpu: &mut Cpu, addr: u16) -> Result<()> {
    push_stack(cpu, hi_byte(addr))?;
    push_stack(cpu, lo_byte(addr))?;
    Ok(())
}

fn pop_stack(cpu: &mut Cpu) -> Result<u8> {
    let sp = (cpu.reg.sp as u16) + STACK_OFFSET;
    let val = cpu.bus.read(sp)?;
    cpu.reg.sp += 1;
    Ok(val)
}

fn pop_stack_addr(cpu: &mut Cpu) -> Result<u16> {
    Ok(make_address(pop_stack(cpu)?, pop_stack(cpu)?))
}

fn branch_on_predicate(am: AddressingMode, cpu: &mut Cpu, take_branch: bool) -> Result<u8> {
    let cross = cross_page_boundary(am, cpu);

    if take_branch {
        let addr = deref_address(am, cpu)?;
        cpu.reg.pc = addr;
        if cross {
            Ok(2)
        } else {
            Ok(1)
        }
    } else {
        // branch not taken
        Ok(0)
    }
}

fn compare(am: AddressingMode, cpu: &mut Cpu, val: u8) -> Result<u8> {
    let cross = cross_page_boundary(am, cpu);
    let m = deref_byte(am, cpu)?;
    let diff = val - m;

    cpu.reg.status.set(StatusFlags::NEGATIVE, diff & 0x80 != 0);
    cpu.reg.status.set(StatusFlags::ZERO, m == val);
    cpu.reg.status.set(StatusFlags::CARRY, val >= m);
    if cross {
        Ok(1)
    } else {
        Ok(0)
    }
}

fn adc(am: AddressingMode, cpu: &mut Cpu) -> Result<u8> {
    let m = deref_byte(am, cpu)?;
    let crossed = cross_page_boundary(am, cpu);
    let c = if cpu.reg.status.contains(StatusFlags::CARRY) {
        1u8
    } else {
        0u8
    };
    let a = cpu.reg.a;
    let sum = (a as u16) + (m as u16) + (c as u16);

    let masked = (sum & 0xFF) as u8;
    cpu.reg.a = masked;

    // https://www.righto.com/2012/12/the-6502-overflow-flag-explained.html
    set_negative_flag(cpu, masked);
    set_zero_flag(cpu, masked);
    let carry = sum & 0xFF00 != 0;
    cpu.reg.status.set(StatusFlags::CARRY, carry);
    cpu.reg.status.set(
        StatusFlags::OVERFLOW,
        (is_negative(m) == is_negative(a)) && (is_negative(masked) != is_negative(a)),
    );

    if crossed {
        Ok(1)
    } else {
        Ok(0)
    }
}

fn and(am: AddressingMode, cpu: &mut Cpu) -> Result<u8> {
    let a = cpu.reg.a;
    let m = deref_byte(am, cpu)?;
    let crossed = cross_page_boundary(am, cpu);
    cpu.reg.a = a & m;
    set_negative_flag(cpu, cpu.reg.a);
    set_zero_flag(cpu, cpu.reg.a);

    if crossed {
        Ok(1)
    } else {
        Ok(0)
    }
}

fn asl(am: AddressingMode, cpu: &mut Cpu) -> Result<u8> {
    if matches!(am, AddressingMode::Accumulator) {
        let carry = is_negative(cpu.reg.a);
        cpu.reg.a <<= 1;
        cpu.reg.status.set(StatusFlags::CARRY, carry);
        set_zero_flag(cpu, cpu.reg.a);
        set_negative_flag(cpu, cpu.reg.a);
    } else {
        let addr = effective_addr(am, cpu)?;
        let val = deref_byte(am, cpu)?;
        let carry = is_negative(val);
        cpu.bus.write(addr, val << 1)?;
        cpu.reg.status.set(StatusFlags::CARRY, carry);
        set_zero_flag(cpu, val << 1);
        set_negative_flag(cpu, val << 1);
    }
    Ok(0)
}

fn bcc(am: AddressingMode, cpu: &mut Cpu) -> Result<u8> {
    branch_on_predicate(am, cpu, !cpu.reg.status.contains(StatusFlags::CARRY))
}

fn bcs(am: AddressingMode, cpu: &mut Cpu) -> Result<u8> {
    branch_on_predicate(am, cpu, cpu.reg.status.contains(StatusFlags::CARRY))
}

fn beq(am: AddressingMode, cpu: &mut Cpu) -> Result<u8> {
    branch_on_predicate(am, cpu, cpu.reg.status.contains(StatusFlags::ZERO))
}

fn bit(am: AddressingMode, cpu: &mut Cpu) -> Result<u8> {
    let m = deref_byte(am, cpu)?;
    cpu.reg
        .status
        .set(StatusFlags::NEGATIVE, m & 0b10000000 != 0);
    cpu.reg
        .status
        .set(StatusFlags::OVERFLOW, m & 0b01000000 != 0);
    set_zero_flag(cpu, cpu.reg.a & m);
    Ok(0)
}

fn bmi(am: AddressingMode, cpu: &mut Cpu) -> Result<u8> {
    branch_on_predicate(am, cpu, cpu.reg.status.contains(StatusFlags::NEGATIVE))
}

fn bne(am: AddressingMode, cpu: &mut Cpu) -> Result<u8> {
    branch_on_predicate(am, cpu, !cpu.reg.status.contains(StatusFlags::ZERO))
}

fn bpl(am: AddressingMode, cpu: &mut Cpu) -> Result<u8> {
    branch_on_predicate(am, cpu, !cpu.reg.status.contains(StatusFlags::NEGATIVE))
}

fn brk(_am: AddressingMode, cpu: &mut Cpu) -> Result<u8> {
    let mut pc = cpu.reg.pc;
    pc += 2;
    push_stack_addr(cpu, pc)?;

    let mut flags = cpu.reg.status.clone();
    flags.insert(StatusFlags::BREAK);
    push_stack(cpu, cpu.reg.status.bits())?;

    cpu.reg.status.insert(StatusFlags::INTERRUPT_DISABLE);
    cpu.interrupt = Some(Interrupt::Request);

    Ok(0)
}

fn bvc(am: AddressingMode, cpu: &mut Cpu) -> Result<u8> {
    branch_on_predicate(am, cpu, !cpu.reg.status.contains(StatusFlags::OVERFLOW))
}

fn bvs(am: AddressingMode, cpu: &mut Cpu) -> Result<u8> {
    branch_on_predicate(am, cpu, cpu.reg.status.contains(StatusFlags::OVERFLOW))
}

fn clc(_am: AddressingMode, cpu: &mut Cpu) -> Result<u8> {
    cpu.reg.status.remove(StatusFlags::CARRY);
    Ok(0)
}

fn cld(_am: AddressingMode, cpu: &mut Cpu) -> Result<u8> {
    cpu.reg.status.remove(StatusFlags::DECIMAL);
    Ok(0)
}

fn cli(_am: AddressingMode, cpu: &mut Cpu) -> Result<u8> {
    cpu.reg.status.remove(StatusFlags::INTERRUPT_DISABLE);
    Ok(0)
}

fn clv(_am: AddressingMode, cpu: &mut Cpu) -> Result<u8> {
    cpu.reg.status.remove(StatusFlags::OVERFLOW);
    Ok(0)
}

fn cmp(am: AddressingMode, cpu: &mut Cpu) -> Result<u8> {
    compare(am, cpu, cpu.reg.a)
}

fn cpx(am: AddressingMode, cpu: &mut Cpu) -> Result<u8> {
    compare(am, cpu, cpu.reg.x)
}

fn cpy(am: AddressingMode, cpu: &mut Cpu) -> Result<u8> {
    compare(am, cpu, cpu.reg.y)
}

fn dec(am: AddressingMode, cpu: &mut Cpu) -> Result<u8> {
    let e = effective_addr(am, cpu)?;
    let val = deref_byte(am, cpu)? - 1;
    cpu.bus.write(e, val)?;
    set_negative_flag(cpu, val);
    set_zero_flag(cpu, val);
    Ok(0)
}

fn dex(_am: AddressingMode, cpu: &mut Cpu) -> Result<u8> {
    cpu.reg.x -= 1;
    set_negative_flag(cpu, cpu.reg.x);
    set_zero_flag(cpu, cpu.reg.x);
    Ok(0)
}

fn dey(_am: AddressingMode, cpu: &mut Cpu) -> Result<u8> {
    cpu.reg.y -= 1;
    set_negative_flag(cpu, cpu.reg.y);
    set_zero_flag(cpu, cpu.reg.y);
    Ok(0)
}

fn eor(am: AddressingMode, cpu: &mut Cpu) -> Result<u8> {
    let cross = cross_page_boundary(am, cpu);
    let m = deref_byte(am, cpu)?;

    cpu.reg.a ^= m;

    set_negative_flag(cpu, cpu.reg.a);
    set_zero_flag(cpu, cpu.reg.a);
    if cross {
        Ok(1)
    } else {
        Ok(0)
    }
}

fn inc(am: AddressingMode, cpu: &mut Cpu) -> Result<u8> {
    let addr = effective_addr(am, cpu)?;
    let val = cpu.bus.read(addr)? + 1;
    cpu.bus.write(addr, val)?;

    set_negative_flag(cpu, val);
    set_zero_flag(cpu, val);
    Ok(0)
}

fn inx(_: AddressingMode, cpu: &mut Cpu) -> Result<u8> {
    cpu.reg.x += 1;
    set_negative_flag(cpu, cpu.reg.x);
    set_zero_flag(cpu, cpu.reg.x);
    Ok(0)
}

fn iny(_: AddressingMode, cpu: &mut Cpu) -> Result<u8> {
    cpu.reg.y += 1;
    set_negative_flag(cpu, cpu.reg.y);
    set_zero_flag(cpu, cpu.reg.y);
    Ok(0)
}

fn jmp(am: AddressingMode, cpu: &mut Cpu) -> Result<u8> {
    cpu.reg.pc = deref_address(am, cpu)?;
    Ok(0)
}

fn jsr(am: AddressingMode, cpu: &mut Cpu) -> Result<u8> {
    push_stack_addr(cpu, cpu.reg.pc + 2)?;
    cpu.reg.pc = deref_address(am, cpu)?;
    Ok(0)
}

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

fn ldx(am: AddressingMode, cpu: &mut Cpu) -> Result<u8> {
    cpu.reg.x = deref_byte(am, cpu)?;
    let crossed = cross_page_boundary(am, cpu);

    set_negative_flag(cpu, cpu.reg.x);
    set_zero_flag(cpu, cpu.reg.x);

    if crossed {
        Ok(1)
    } else {
        Ok(0)
    }
}

fn ldy(am: AddressingMode, cpu: &mut Cpu) -> Result<u8> {
    cpu.reg.y = deref_byte(am, cpu)?;
    let crossed = cross_page_boundary(am, cpu);

    set_negative_flag(cpu, cpu.reg.y);
    set_zero_flag(cpu, cpu.reg.y);

    if crossed {
        Ok(1)
    } else {
        Ok(0)
    }
}

fn lsr(am: AddressingMode, cpu: &mut Cpu) -> Result<u8> {
    if matches!(am, AddressingMode::Accumulator) {
        let carry = cpu.reg.a & 0b1 == 1;
        cpu.reg.a >>= 1;
        cpu.reg.status.set(StatusFlags::CARRY, carry);
        set_zero_flag(cpu, cpu.reg.a);
        cpu.reg.status.remove(StatusFlags::NEGATIVE);
    } else {
        let addr = effective_addr(am, cpu)?;
        let val = deref_byte(am, cpu)?;
        let carry = val & 0b1 == 1;
        cpu.bus.write(addr, val >> 1)?;

        cpu.reg.status.set(StatusFlags::CARRY, carry);
        set_zero_flag(cpu, val >> 1);
        cpu.reg.status.remove(StatusFlags::NEGATIVE);
    }
    Ok(0)
}

fn nop(_: AddressingMode, _: &mut Cpu) -> Result<u8> {
    Ok(0)
}

fn ora(am: AddressingMode, cpu: &mut Cpu) -> Result<u8> {
    let cross = cross_page_boundary(am, cpu);
    let m = deref_byte(am, cpu)?;
    cpu.reg.a |= m;

    set_negative_flag(cpu, cpu.reg.a);
    set_zero_flag(cpu, cpu.reg.a);
    if cross {
        Ok(1)
    } else {
        Ok(0)
    }
}

fn pha(_: AddressingMode, cpu: &mut Cpu) -> Result<u8> {
    push_stack(cpu, cpu.reg.a)?;
    Ok(0)
}

fn php(_: AddressingMode, cpu: &mut Cpu) -> Result<u8> {
    let mut flags = cpu.reg.status.clone();
    flags.insert(StatusFlags::BREAK);
    flags.insert(StatusFlags::UNUSED);
    push_stack(cpu, flags.bits())?;
    Ok(0)
}

fn pla(_: AddressingMode, cpu: &mut Cpu) -> Result<u8> {
    cpu.reg.a = pop_stack(cpu)?;
    set_zero_flag(cpu, cpu.reg.a);
    set_negative_flag(cpu, cpu.reg.a);
    Ok(0)
}

fn plp(_: AddressingMode, cpu: &mut Cpu) -> Result<u8> {
    // Invalid flags should never happen, we use all 8 bits
    cpu.reg.status =
        StatusFlags::from_bits(pop_stack(cpu)?).expect("Flags popped from stack invalid.");
    Ok(0)
}

#[cfg(test)]
mod exec_tests {
    use super::AddressingMode::*;
    use crate::cpu::cpu::Cpu;

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
