use std::{error::Error, ops::Add};

use super::{isa::{Instr, AddressingMode}, reg::StatusFlags};
use crate::{cpu::cpu::Cpu};

type Result<T> = core::result::Result<T, Box<dyn Error>>;

// Returns number of cycles this instruction needs
fn exec_instr(i: Instr, cpu: &mut Cpu) -> Result<u8> {
    Ok(0)
}

fn make_address(lo: u8, hi: u8) -> u16 {
    ((hi as u16) << 8) & 0xFF00 | ((lo as u16) & 0xFF)
}

fn deref_byte(am: AddressingMode, cpu: &Cpu) -> Result<u8> {
    use AddressingMode::*;
    match am {
        Accumulator => Ok(cpu.reg.a),
        Absolute(addr) => cpu.bus.read(addr),
        AbsoluteX(addr) => cpu.bus.read(addr + cpu.reg.x as u16),
        AbsoluteY(addr) => cpu.bus.read(addr + cpu.reg.y as u16),
        Immediate(val) => Ok(val),
        Implied => panic!("deref Implied"),
        Indirect(_) => panic!("deref Indirect"),
        XIndirect(offset) => {
            let lo = cpu.bus.read(((offset + cpu.reg.x) & 0xFF) as u16)?;
            let hi = cpu.bus.read(((offset + cpu.reg.x + 1) & 0xFF) as u16)?;
            let effective_addr = make_address(lo, hi);
            println!("effective addr: {:X}", effective_addr);
            cpu.bus.read(effective_addr)
        }
        IndirectY(offset) => {
            let lo = cpu.bus.read(offset as u16)?;
            let hi = cpu.bus.read((offset + 1) as u16)?;
            let effective = make_address(lo, hi);
            cpu.bus.read(effective + (cpu.reg.y as u16))
        }
        Relative(_) => panic!("deref Relative"),
        ZeroPage(offset) => cpu.bus.read(offset as u16),
        ZeroPageX(offset) => cpu.bus.read(((offset as u16) + (cpu.reg.x as u16)) & 0xFF),
        ZeroPageY(offset) => cpu.bus.read(((offset as u16) + (cpu.reg.y as u16)) & 0xFF),
    }
}

fn deref_address(am: AddressingMode, cpu: &Cpu) -> Result<u16> {
    use AddressingMode::*;
    match am {
        Indirect(addr) => {
            let lo = cpu.bus.read(addr)?;
            let hi = cpu.bus.read(addr + 1)?;
            Ok(make_address(lo, hi))
        }
        Absolute(addr) => Ok(addr),
        Relative(offset) => {
            if is_negative(offset) {
                Ok(cpu.reg.pc - (offset.wrapping_neg() as u16))
            } else {
                Ok(cpu.reg.pc + (offset as u16))
            }
        },
        _ => panic!("deref_address on {:?}", am)
    }
}

fn is_negative(b: u8) -> bool {
    b & 0x80 != 0
}

fn page_num(addr: u16) -> u16 {
    addr & 0xF0
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
        Relative(offset) => {
            page_num(cpu.reg.pc) != page_num(cpu.reg.pc + (offset as u16))
        }
        _ => false
    }
}

// Returns extra # of cycles needed to run (this extra will be added to number
// of cycles in instruction matrix)
fn lda(am: AddressingMode, cpu: &mut Cpu) -> Result<u8> {
    cpu.reg.a = deref_byte(am, cpu)?;

    cpu.reg.status.set(StatusFlags::NEGATIVE, is_negative(cpu.reg.a));
    cpu.reg.status.set(StatusFlags::ZERO, cpu.reg.a == 0);

    if cross_page_boundary(am, cpu) {
        Ok(1)
    } else {
        Ok(0)
    }
}

#[cfg(test)]
mod addressing_mode_tests {
    use crate::cpu::cpu::Cpu;
    use super::AddressingMode::*;

    use super::deref_address;
    use super::deref_byte;

    // Some test cases come from examples in this post:
    // http://www.emulator101.com/6502-addressing-modes.html

    #[test]
    fn accumulator() {
        let mut cpu = Cpu::mock(&[]);
        cpu.reg.a = 42;
        assert_eq!(deref_byte(Accumulator, &cpu).unwrap(), 42);
    }

    #[test]
    fn absolute_byte() {
        let cpu = Cpu::mock(&[0xFE, 0xCA, 0xEF, 0xBE]);
        assert_eq!(deref_byte(Absolute(0x0000), &cpu).unwrap(), 0xFE);
        assert_eq!(deref_byte(Absolute(0x0002), &cpu).unwrap(), 0xEF);
    }

    #[test]
    fn absolute_x_index() {
        let mut cpu = Cpu::mock(&[0; 0x1FFF]);
        // no carry
        cpu.bus.write(0x08F7, 0x42).unwrap();
        cpu.reg.x = 0x7;
        assert_eq!(deref_byte(AbsoluteX(0x08F0), &cpu).unwrap(), 0x42);

        // carry
        cpu.bus.write(0x0200, 0x17).unwrap();
        cpu.reg.x = 0xFF;
        assert_eq!(deref_byte(AbsoluteX(0x101), &cpu).unwrap(), 0x17);
    }

    #[test]
    fn absolute_y_index() {
        let mut cpu = Cpu::mock(&[0; 0x1FFF]);
        // no carry
        cpu.bus.write(0x08F7, 0x42).unwrap();
        cpu.reg.y = 0x7;
        assert_eq!(deref_byte(AbsoluteY(0x08F0), &cpu).unwrap(), 0x42);

        // carry
        cpu.bus.write(0x0200, 0x17).unwrap();
        cpu.reg.y = 0xFF;
        assert_eq!(deref_byte(AbsoluteY(0x101), &cpu).unwrap(), 0x17);
    }

    #[test]
    fn immediate() {
        let cpu = Cpu::mock(&[]);
        assert_eq!(deref_byte(Immediate(0x25), &cpu).unwrap(), 0x25);
        assert_eq!(deref_byte(Immediate(0xFF), &cpu).unwrap(), 0xFF);
        assert_eq!(deref_byte(Immediate(0x00), &cpu).unwrap(), 0x00);
    }

    #[test]
    fn x_indirect() {
        let mut cpu = Cpu::mock(&[0; 0x1FFF]);
        cpu.reg.x = 0x04;
        cpu.bus.write(0x0024, 0x74).unwrap();
        cpu.bus.write(0x0025, 0x10).unwrap();
        cpu.bus.write(0x1074, 0x42).unwrap();
        assert_eq!(deref_byte(XIndirect(0x20), &cpu).unwrap(), 0x42);
    }

    #[test]
    fn indirect_y() {
        let mut cpu = Cpu::mock(&[0; 0x1FFF]);
        cpu.reg.y = 0x10;
        cpu.bus.write(0x74, 0x28).unwrap();
        cpu.bus.write(0x75, 0x10).unwrap();
        cpu.bus.write(0x1038, 0x42).unwrap();
        assert_eq!(deref_byte(IndirectY(0x74), &cpu).unwrap(), 0x42);
    }

    #[test]
    fn zero_page() {
        let mut cpu = Cpu::mock(&[0; 0x1FFF]);
        cpu.bus.write(0x00, 0xAB).unwrap();
        cpu.bus.write(0x22, 0xCD).unwrap();
        cpu.bus.write(0x33, 0xEF).unwrap();
        cpu.bus.write(0xFF, 0x77).unwrap();
        assert_eq!(deref_byte(ZeroPage(0x00), &cpu).unwrap(), 0xAB);
        assert_eq!(deref_byte(ZeroPage(0x22), &cpu).unwrap(), 0xCD);
        assert_eq!(deref_byte(ZeroPage(0x33), &cpu).unwrap(), 0xEF);
        assert_eq!(deref_byte(ZeroPage(0xFF), &cpu).unwrap(), 0x77);
    }

    #[test]
    fn zero_page_x() {
        let mut cpu = Cpu::mock(&[0; 0x1FFF]);
        cpu.bus.write(0x20, 0x42).unwrap();
        cpu.reg.x = 0x60;

        // carry ignored
        // 0x60 + 0xC0 = 0x120 -> 0x20
        assert_eq!(deref_byte(ZeroPageX(0xC0), &cpu).unwrap(), 0x42);

        // no carry
        cpu.reg.x = 0x01;
        assert_eq!(deref_byte(ZeroPageX(0x1F), &cpu).unwrap(), 0x42);
    }


    #[test]
    fn zero_page_y() {
        let mut cpu = Cpu::mock(&[0; 0x1FFF]);
        cpu.bus.write(0x20, 0x42).unwrap();
        cpu.reg.y = 0x60;

        // carry ignored
        // 0x60 + 0xC0 = 0x120 -> 0x20
        assert_eq!(deref_byte(ZeroPageY(0xC0), &cpu).unwrap(), 0x42);

        // no carry
        cpu.reg.y = 0x01;
        assert_eq!(deref_byte(ZeroPageY(0x1F), &cpu).unwrap(), 0x42);
    }

    #[test]
    fn relative() {
        let mut cpu = Cpu::mock(&[]);
        
        cpu.reg.pc = 0x100;
        // Positive offsets
        assert_eq!(deref_address(Relative(0x8), &cpu).unwrap(), 0x108);
        assert_eq!(deref_address(Relative(0x7F), &cpu).unwrap(), 0x17F);

        // Negative offsets
        assert_eq!(deref_address(Relative(0xFF), &cpu).unwrap(), 0x0FF);
        assert_eq!(deref_address(Relative(0x9C), &cpu).unwrap(), 0x9C);

        cpu.reg.pc = 0x1234;
        // 0x1234 - 0x4 = 0x1230
        assert_eq!(deref_address(Relative(0xFC), &cpu).unwrap(), 0x1230);
        // 0x1234 - 0x28 = 0x120C
        assert_eq!(deref_address(Relative(0xD8), &cpu).unwrap(), 0x120C);
        // 0x1234 - 0x80 = 0x11B4
        assert_eq!(deref_address(Relative(0x80), &cpu).unwrap(), 0x11B4);
    }

    #[test]
    fn absolute_addr() {
        let cpu = Cpu::mock(&[]);
        assert_eq!(deref_address(Absolute(0xCAFE), &cpu).unwrap(), 0xCAFE);
        assert_eq!(deref_address(Absolute(0xBEA7), &cpu).unwrap(), 0xBEA7);
    }

    #[test]
    fn indirect() {
        let mut cpu = Cpu::mock(&[0; 0x1FFF]);
        cpu.bus.write(0x1000, 0x52).unwrap();
        cpu.bus.write(0x1001, 0x3a).unwrap();
        assert_eq!(deref_address(Indirect(0x1000), &cpu).unwrap(), 0x3a52);

        cpu.bus.write(0xFF, 0x76).unwrap();
        cpu.bus.write(0x100, 0x17).unwrap();
        assert_eq!(deref_address(Indirect(0xFF), &cpu).unwrap(), 0x1776);
    }

}