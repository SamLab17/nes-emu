use std::fmt;
use std::num::Wrapping;
use std::error::Error;

use crate::error::Result;

use super::cpu::STACK_OFFSET;
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
type OpcodeFn = fn(AddressingMode, &mut Cpu) -> Result<u16>;

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
        LAX => lax,
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
        ROL => rol,
        ROR => ror,
        RTI => rti,
        RTS => rts,
        SBC => sbc,
        SEC => sec,
        SED => sed,
        SEI => sei,
        STA => sta,
        STX => stx,
        STY => sty,
        TAX => tax,
        TAY => tay,
        TSX => tsx,
        TXA => txa,
        TXS => txs,
        TYA => tya,
        INVALID => invalid_op,
    }
}


pub fn exec_instr(i: Instr, cpu: &mut Cpu) -> Result<u16> {
    // Lookup opcode function
    // Run instruction, return # of extra cycles needed
    lookup_opcode_fn(i.op)(i.mode, cpu)
}

fn invalid_op(am: AddressingMode, _: &mut Cpu) -> Result<u16> {
    Err(Box::new(ExecutionError::InvalidInstruction(Instr {
        op: Opcode::INVALID,
        mode: am,
    })))
}

// Returns whether this addressing mode will cross a page boundary
// (which typically incurs an extra cycle cost)
fn cross_page_boundary(am: AddressingMode, cpu: &mut Cpu) -> bool {
    use AddressingMode::*;
    match am {
        AbsoluteX(addr) => page_num(addr) != page_num(addr + (cpu.reg.x as u16)),
        AbsoluteY(addr) => page_num(addr) != page_num(addr.wrapping_add(cpu.reg.y as u16)),
        IndirectY(offset) => {
            let lo = cpu.read(offset as u16).unwrap();
            let hi = cpu.read((offset.wrapping_add(1)) as u16).unwrap();
            let effective = make_address(lo, hi);

            page_num(effective) != page_num(effective.wrapping_add(cpu.reg.y as u16))
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

pub fn push_stack(cpu: &mut Cpu, byte: u8) -> Result<()> {
    let sp = (cpu.reg.sp as u16) + STACK_OFFSET;
    cpu.write(sp, byte)?;
    // println!("push_stack: sp: {:X}, val: {:X}", cpu.reg.sp, byte);
    cpu.reg.sp -= 1;
    Ok(())
}

pub fn push_stack_addr(cpu: &mut Cpu, addr: u16) -> Result<()> {
    push_stack(cpu, hi_byte(addr))?;
    push_stack(cpu, lo_byte(addr))?;
    Ok(())
}

pub fn pop_stack(cpu: &mut Cpu) -> Result<u8> {
    cpu.reg.sp += 1;
    let sp = (cpu.reg.sp as u16) + STACK_OFFSET;
    let val = cpu.read(sp)?;
    // println!("pop stack: sp: {:X}, val: {:X}", cpu.reg.sp, val);
    Ok(val)
}

pub fn pop_stack_addr(cpu: &mut Cpu) -> Result<u16> {
    let lo = pop_stack(cpu)?;
    let hi = pop_stack(cpu)?;
    Ok(make_address(lo, hi))
}

fn branch_on_predicate(am: AddressingMode, cpu: &mut Cpu, take_branch: bool) -> Result<u16> {
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

fn compare(am: AddressingMode, cpu: &mut Cpu, val: u8) -> Result<u16> {
    let cross = cross_page_boundary(am, cpu);
    let m = deref_byte(am, cpu)?;
    let diff = (Wrapping(val) - Wrapping(m)).0;

    cpu.reg.status.set(StatusFlags::NEGATIVE, diff & 0x80 != 0);
    cpu.reg.status.set(StatusFlags::ZERO, m == val);
    cpu.reg.status.set(StatusFlags::CARRY, val >= m);
    if cross {
        Ok(1)
    } else {
        Ok(0)
    }
}

fn adc(am: AddressingMode, cpu: &mut Cpu) -> Result<u16> {
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

fn and(am: AddressingMode, cpu: &mut Cpu) -> Result<u16> {
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

fn asl(am: AddressingMode, cpu: &mut Cpu) -> Result<u16> {
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
        cpu.write(addr, val << 1)?;
        cpu.reg.status.set(StatusFlags::CARRY, carry);
        set_zero_flag(cpu, val << 1);
        set_negative_flag(cpu, val << 1);
    }
    Ok(0)
}

fn bcc(am: AddressingMode, cpu: &mut Cpu) -> Result<u16> {
    branch_on_predicate(am, cpu, !cpu.reg.status.contains(StatusFlags::CARRY))
}

fn bcs(am: AddressingMode, cpu: &mut Cpu) -> Result<u16> {
    branch_on_predicate(am, cpu, cpu.reg.status.contains(StatusFlags::CARRY))
}

fn beq(am: AddressingMode, cpu: &mut Cpu) -> Result<u16> {
    branch_on_predicate(am, cpu, cpu.reg.status.contains(StatusFlags::ZERO))
}

fn bit(am: AddressingMode, cpu: &mut Cpu) -> Result<u16> {
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

fn bmi(am: AddressingMode, cpu: &mut Cpu) -> Result<u16> {
    branch_on_predicate(am, cpu, cpu.reg.status.contains(StatusFlags::NEGATIVE))
}

fn bne(am: AddressingMode, cpu: &mut Cpu) -> Result<u16> {
    branch_on_predicate(am, cpu, !cpu.reg.status.contains(StatusFlags::ZERO))
}

fn bpl(am: AddressingMode, cpu: &mut Cpu) -> Result<u16> {
    branch_on_predicate(am, cpu, !cpu.reg.status.contains(StatusFlags::NEGATIVE))
}

fn brk(_am: AddressingMode, cpu: &mut Cpu) -> Result<u16> {
    // panic!("BRK");
    push_stack_addr(cpu, cpu.reg.pc + 2)?;

    cpu.reg.status.insert(StatusFlags::INTERRUPT_DISABLE);

    let mut flags = cpu.reg.status.clone();
    flags.insert(StatusFlags::BREAK);
    push_stack(cpu, cpu.reg.status.bits())?;

    cpu.reg.pc = make_address(cpu.read(0xFFFE)?, cpu.read(0xFFFF)?);
    // No need to set interrupt field here, we've already handled it
    // cpu.interrupt = Some(super::cpu::Interrupt::Request);

    Ok(0)
}

fn bvc(am: AddressingMode, cpu: &mut Cpu) -> Result<u16> {
    branch_on_predicate(am, cpu, !cpu.reg.status.contains(StatusFlags::OVERFLOW))
}

fn bvs(am: AddressingMode, cpu: &mut Cpu) -> Result<u16> {
    branch_on_predicate(am, cpu, cpu.reg.status.contains(StatusFlags::OVERFLOW))
}

fn clc(_am: AddressingMode, cpu: &mut Cpu) -> Result<u16> {
    cpu.reg.status.remove(StatusFlags::CARRY);
    Ok(0)
}

fn cld(_am: AddressingMode, cpu: &mut Cpu) -> Result<u16> {
    cpu.reg.status.remove(StatusFlags::DECIMAL);
    Ok(0)
}

fn cli(_am: AddressingMode, cpu: &mut Cpu) -> Result<u16> {
    cpu.reg.status.remove(StatusFlags::INTERRUPT_DISABLE);
    Ok(0)
}

fn clv(_am: AddressingMode, cpu: &mut Cpu) -> Result<u16> {
    cpu.reg.status.remove(StatusFlags::OVERFLOW);
    Ok(0)
}

fn cmp(am: AddressingMode, cpu: &mut Cpu) -> Result<u16> {
    compare(am, cpu, cpu.reg.a)
}

fn cpx(am: AddressingMode, cpu: &mut Cpu) -> Result<u16> {
    compare(am, cpu, cpu.reg.x)
}

fn cpy(am: AddressingMode, cpu: &mut Cpu) -> Result<u16> {
    compare(am, cpu, cpu.reg.y)
}

fn dec(am: AddressingMode, cpu: &mut Cpu) -> Result<u16> {
    let e = effective_addr(am, cpu)?;
    let val = deref_byte(am, cpu)?.wrapping_sub(1);
    cpu.write(e, val)?;
    set_negative_flag(cpu, val);
    set_zero_flag(cpu, val);
    Ok(0)
}

fn dex(_am: AddressingMode, cpu: &mut Cpu) -> Result<u16> {
    cpu.reg.x = cpu.reg.x.wrapping_sub(1);
    set_negative_flag(cpu, cpu.reg.x);
    set_zero_flag(cpu, cpu.reg.x);
    Ok(0)
}

fn dey(_am: AddressingMode, cpu: &mut Cpu) -> Result<u16> {
    cpu.reg.y = cpu.reg.y.wrapping_sub(1);
    set_negative_flag(cpu, cpu.reg.y);
    set_zero_flag(cpu, cpu.reg.y);
    Ok(0)
}

fn eor(am: AddressingMode, cpu: &mut Cpu) -> Result<u16> {
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

fn inc(am: AddressingMode, cpu: &mut Cpu) -> Result<u16> {
    let addr = effective_addr(am, cpu)?;
    let val = cpu.read(addr)?.wrapping_add(1);
    cpu.write(addr, val)?;

    set_negative_flag(cpu, val);
    set_zero_flag(cpu, val);
    Ok(0)
}

fn inx(_: AddressingMode, cpu: &mut Cpu) -> Result<u16> {
    cpu.reg.x = cpu.reg.x.wrapping_add(1);
    set_negative_flag(cpu, cpu.reg.x);
    set_zero_flag(cpu, cpu.reg.x);
    Ok(0)
}

fn iny(_: AddressingMode, cpu: &mut Cpu) -> Result<u16> {
    cpu.reg.y = cpu.reg.y.wrapping_add(1);
    set_negative_flag(cpu, cpu.reg.y);
    set_zero_flag(cpu, cpu.reg.y);
    Ok(0)
}

fn jmp(am: AddressingMode, cpu: &mut Cpu) -> Result<u16> {
    cpu.reg.pc = deref_address(am, cpu)?;
    Ok(0)
}

fn jsr(am: AddressingMode, cpu: &mut Cpu) -> Result<u16> {
    push_stack_addr(cpu, cpu.reg.pc - 1)?;
    cpu.reg.pc = deref_address(am, cpu)?;
    Ok(0)
}

fn lax(am: AddressingMode, cpu: &mut Cpu) -> Result<u16> {
    let crossed = cross_page_boundary(am, cpu);
    let m = deref_byte(am, cpu)?;
    cpu.reg.x = m;
    cpu.reg.a = m;
    set_negative_flag(cpu, m);
    set_zero_flag(cpu, m);
    if crossed {
        Ok(1)
    } else {
        Ok(0)
    }
}

fn lda(am: AddressingMode, cpu: &mut Cpu) -> Result<u16> {
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

fn ldx(am: AddressingMode, cpu: &mut Cpu) -> Result<u16> {
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

fn ldy(am: AddressingMode, cpu: &mut Cpu) -> Result<u16> {
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

fn lsr(am: AddressingMode, cpu: &mut Cpu) -> Result<u16> {
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
        cpu.write(addr, val >> 1)?;

        cpu.reg.status.set(StatusFlags::CARRY, carry);
        set_zero_flag(cpu, val >> 1);
        cpu.reg.status.remove(StatusFlags::NEGATIVE);
    }
    Ok(0)
}

fn nop(am: AddressingMode, cpu: &mut Cpu) -> Result<u16> {
    if cross_page_boundary(am, cpu) {
        Ok(1)
    } else {
        Ok(0)
    }
}

fn ora(am: AddressingMode, cpu: &mut Cpu) -> Result<u16> {
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

fn pha(_: AddressingMode, cpu: &mut Cpu) -> Result<u16> {
    push_stack(cpu, cpu.reg.a)?;
    Ok(0)
}

fn php(_: AddressingMode, cpu: &mut Cpu) -> Result<u16> {
    let mut flags = cpu.reg.status.clone();
    flags.insert(StatusFlags::BREAK);
    flags.insert(StatusFlags::UNUSED);
    push_stack(cpu, flags.bits())?;
    Ok(0)
}

fn pla(_: AddressingMode, cpu: &mut Cpu) -> Result<u16> {
    cpu.reg.a = pop_stack(cpu)?;
    set_zero_flag(cpu, cpu.reg.a);
    set_negative_flag(cpu, cpu.reg.a);
    Ok(0)
}

fn plp(_: AddressingMode, cpu: &mut Cpu) -> Result<u16> {
    // Invalid flags should never happen, we use all 8 bits

    cpu.reg.status =
        StatusFlags::from_bits(pop_stack(cpu)?).expect("Flags popped from stack invalid.");
    cpu.reg.status.remove(StatusFlags::BREAK);
    cpu.reg.status.insert(StatusFlags::UNUSED);
    // cpu.reg.status.remove(StatusFlags::UNUSED);
    Ok(0)
}

fn rol(am: AddressingMode, cpu: &mut Cpu) -> Result<u16> {
    let carry = if cpu.reg.status.contains(StatusFlags::CARRY) {
        1
    } else {
        0
    };
    if matches!(am, AddressingMode::Accumulator) {
        let a = cpu.reg.a;
        let new_carry = is_negative(a);
        let rot = ((a as u16) << 1) | carry;
        let masked = (rot & 0xFF) as u8;
        cpu.reg.status.set(StatusFlags::CARRY, new_carry);
        set_negative_flag(cpu, masked);
        set_zero_flag(cpu, masked);

        cpu.reg.a = masked;
    } else {
        let addr = effective_addr(am, cpu)?;
        let m = deref_byte(am, cpu)?;
        let new_carry = is_negative(m);
        let rot = ((m as u16) << 1) | carry;
        let masked = (rot & 0xFF) as u8;

        cpu.reg.status.set(StatusFlags::CARRY, new_carry);
        set_negative_flag(cpu, masked);
        set_zero_flag(cpu, masked);
        cpu.write(addr, masked)?;
    }
    Ok(0)
}

fn ror(am: AddressingMode, cpu: &mut Cpu) -> Result<u16> {
    let carry = if cpu.reg.status.contains(StatusFlags::CARRY) {
        0x80
    } else {
        0
    };
    if matches!(am, AddressingMode::Accumulator) {
        let a = cpu.reg.a;
        let new_carry = a & 1 == 1;
        let rot = ((a as u16) >> 1) | carry;
        let masked = (rot & 0xFF) as u8;

        cpu.reg.status.set(StatusFlags::CARRY, new_carry);
        set_negative_flag(cpu, masked);
        set_zero_flag(cpu, masked);
        cpu.reg.a = masked;
    } else {
        let addr = effective_addr(am, cpu)?;
        let m = deref_byte(am, cpu)?;
        let new_carry = m & 1 == 1;
        let rot = ((m as u16) >> 1) | carry;
        let masked = (rot & 0xFF) as u8;

        cpu.reg.status.set(StatusFlags::CARRY, new_carry);
        set_negative_flag(cpu, masked);
        set_zero_flag(cpu, masked);
        cpu.write(addr, masked)?;
    }
    Ok(0)
}

fn rti(_: AddressingMode, cpu: &mut Cpu) -> Result<u16> {
    let mut status = StatusFlags::from_bits(pop_stack(cpu)?)
        .expect("Status flags popped from stack are invalid.");
    status.remove(StatusFlags::BREAK);
    status.insert(StatusFlags::UNUSED);
    cpu.reg.status = status;

    cpu.reg.pc = pop_stack_addr(cpu)?;
    Ok(0)
}

fn rts(_: AddressingMode, cpu: &mut Cpu) -> Result<u16> {
    cpu.reg.pc = pop_stack_addr(cpu)? + 1;
    Ok(0)
}

fn sbc(am: AddressingMode, cpu: &mut Cpu) -> Result<u16> {
    // literally the same as adc() but with m inverted
    // https://stackoverflow.com/a/29224684
    // A - M - ~C = A + -M - ~C = A + ~M + 1 - (1-C) = A + ~M + C
    let m = !(deref_byte(am, cpu)?);
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

fn sec(_: AddressingMode, cpu: &mut Cpu) -> Result<u16> {
    cpu.reg.status.insert(StatusFlags::CARRY);
    Ok(0)
}

fn sed(_: AddressingMode, cpu: &mut Cpu) -> Result<u16> {
    cpu.reg.status.insert(StatusFlags::DECIMAL);
    Ok(0)
}

fn sei(_: AddressingMode, cpu: &mut Cpu) -> Result<u16> {
    cpu.reg.status.insert(StatusFlags::INTERRUPT_DISABLE);
    Ok(0)
}

fn sta(am: AddressingMode, cpu: &mut Cpu) -> Result<u16> {
    let addr = effective_addr(am, cpu)?;
    cpu.write(addr, cpu.reg.a)?;
    Ok(0)
}

fn stx(am: AddressingMode, cpu: &mut Cpu) -> Result<u16> {
    let addr = effective_addr(am, cpu)?;
    cpu.write(addr, cpu.reg.x)?;
    Ok(0)
}

fn sty(am: AddressingMode, cpu: &mut Cpu) -> Result<u16> {
    let addr = effective_addr(am, cpu)?;
    cpu.write(addr, cpu.reg.y)?;
    Ok(0)
}

fn tax(_: AddressingMode, cpu: &mut Cpu) -> Result<u16> {
    cpu.reg.x = cpu.reg.a;
    set_negative_flag(cpu, cpu.reg.x);
    set_zero_flag(cpu, cpu.reg.x);
    Ok(0)
}

fn tay(_: AddressingMode, cpu: &mut Cpu) -> Result<u16> {
    cpu.reg.y = cpu.reg.a;
    set_negative_flag(cpu, cpu.reg.y);
    set_zero_flag(cpu, cpu.reg.y);
    Ok(0)
}

fn tsx(_: AddressingMode, cpu: &mut Cpu) -> Result<u16> {
    cpu.reg.x = cpu.reg.sp;
    set_negative_flag(cpu, cpu.reg.x);
    set_zero_flag(cpu, cpu.reg.x);
    Ok(0)
}

fn txa(_: AddressingMode, cpu: &mut Cpu) -> Result<u16> {
    cpu.reg.a = cpu.reg.x;
    set_negative_flag(cpu, cpu.reg.a);
    set_zero_flag(cpu, cpu.reg.a);
    Ok(0)
}

fn txs(_: AddressingMode, cpu: &mut Cpu) -> Result<u16> {
    cpu.reg.sp = cpu.reg.x;
    Ok(0)
}

fn tya(_: AddressingMode, cpu: &mut Cpu) -> Result<u16> {
    cpu.reg.a = cpu.reg.y;
    set_negative_flag(cpu, cpu.reg.a);
    set_zero_flag(cpu, cpu.reg.a);
    Ok(0)
}

#[cfg(test)]
mod exec_tests {
    use super::AddressingMode::*;
    use crate::cpu::cpu::Cpu;

    use super::cross_page_boundary;

    #[test]
    fn cross_page_boundary_test() {
        let mut cpu = Cpu::mock(None);
        cpu.reg.x = 0xFF;
        cpu.reg.y = 0xFF;
        assert!(cross_page_boundary(AbsoluteX(0x17), &mut cpu));
        assert!(cross_page_boundary(AbsoluteX(0x01), &mut cpu));
        assert!(cross_page_boundary(AbsoluteX(0xFF), &mut cpu));
        assert!(cross_page_boundary(AbsoluteY(0x17), &mut cpu));
        assert!(cross_page_boundary(AbsoluteY(0x01), &mut cpu));
        assert!(cross_page_boundary(AbsoluteY(0xFF), &mut cpu));

        cpu.reg.x = 0x20;
        cpu.reg.y = 0x20;
        assert!(!cross_page_boundary(AbsoluteX(0x17), &mut cpu));
        assert!(cross_page_boundary(AbsoluteX(0xEE), &mut cpu));
        assert!(!cross_page_boundary(AbsoluteX(0x00), &mut cpu));
        assert!(!cross_page_boundary(AbsoluteY(0x17), &mut cpu));
        assert!(cross_page_boundary(AbsoluteY(0xEE), &mut cpu));
        assert!(!cross_page_boundary(AbsoluteY(0x00), &mut cpu));

        cpu.reg.y = 0x80;
        cpu.write(0x42, 0xFF).unwrap();
        cpu.write(0x43, 0x00).unwrap();
        assert!(cross_page_boundary(IndirectY(0x42), &mut cpu));

        cpu.write(0x42, 0x00).unwrap();
        cpu.write(0x43, 0x10).unwrap();
        assert!(!cross_page_boundary(IndirectY(0x42), &mut cpu));

        cpu.reg.pc = 0x01FF;
        // -1 jump
        assert!(!cross_page_boundary(Relative(0xFF), &mut cpu));
        // -64 jump
        assert!(!cross_page_boundary(Relative(0xC0), &mut cpu));
        // + 1 jump
        assert!(cross_page_boundary(Relative(0x1), &mut cpu));
        // + 64 jump
        assert!(cross_page_boundary(Relative(0x40), &mut cpu));

        cpu.reg.pc = 0x0100;
        // -1 jump
        assert!(cross_page_boundary(Relative(0xFF), &mut cpu));
        // -64 jump
        assert!(cross_page_boundary(Relative(0xC0), &mut cpu));
        // + 1 jump
        assert!(!cross_page_boundary(Relative(0x1), &mut cpu));
        // + 64 jump
        assert!(!cross_page_boundary(Relative(0x40), &mut cpu));
    }
}
