use crate::error::Result;
use super::isa::AddressingMode;
use crate::cpu::cpu::Cpu;
use crate::mem::utils::make_address;
use super::utils::is_negative;

pub fn effective_addr(am: AddressingMode, cpu: &Cpu) -> Result<u16> {
    use AddressingMode::*;
    match am {
        Absolute(addr) => Ok(addr),
        AbsoluteX(addr) => Ok(addr + cpu.reg.x as u16),
        AbsoluteY(addr) => Ok(addr + cpu.reg.y as u16),
        XIndirect(offset) => {
            let lo = cpu.bus.read(((offset + cpu.reg.x) & 0xFF) as u16)?;
            let hi = cpu.bus.read(((offset + cpu.reg.x + 1) & 0xFF) as u16)?;
            Ok(make_address(lo, hi))
        },
        IndirectY(offset) => {
            let lo = cpu.bus.read(offset as u16)?;
            let hi = cpu.bus.read((offset + 1) as u16)?;
            Ok(make_address(lo, hi) + (cpu.reg.y as u16))
        },
        ZeroPage(offset) => Ok(offset as u16),
        ZeroPageX(offset) => Ok(((offset as u16) + (cpu.reg.x as u16)) & 0xFF),
        ZeroPageY(offset) => Ok(((offset as u16) + (cpu.reg.y as u16)) & 0xFF),
        Immediate(_) | Relative(_) | Accumulator | Implied | Indirect(_) => panic!("{:?} has no effective address.", am)
    }
}

pub fn deref_byte(am: AddressingMode, cpu: &Cpu) -> Result<u8> {
    use AddressingMode::*;
    match am {
        Accumulator => Ok(cpu.reg.a),
        Immediate(val) => Ok(val),
        Implied | Indirect(_) | Relative(_) => panic!("{:?} cannot be derefenced into a byte value.", am),
        _ => cpu.bus.read(effective_addr(am, cpu)?),
    }
}


pub fn deref_address(am: AddressingMode, cpu: &Cpu) -> Result<u16> {
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