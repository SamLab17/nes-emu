use std::ops::Add;

use crate::error::Result;

use super::{isa::{Instr, AddressingMode}, reg::StatusFlags, deref::effective_addr};
use crate::{cpu::cpu::Cpu};
use crate::mem::utils::{make_address, page_num};
use super::utils::is_negative;

use super::deref::{deref_byte, deref_address};

// Returns number of cycles this instruction needs
pub fn exec_instr(i: Instr, cpu: &mut Cpu) -> Result<u8> {
    Ok(0)
}


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

fn set_negative_flag(cpu: &mut Cpu, val: u8) {
    cpu.reg.status.set(StatusFlags::NEGATIVE, is_negative(val));
}

fn set_zero_flag(cpu: &mut Cpu, val: u8) {
    cpu.reg.status.set(StatusFlags::ZERO, val == 0);
}

// Returns extra # of cycles needed to run (this extra will be added to number
// of cycles in instruction matrix)
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
