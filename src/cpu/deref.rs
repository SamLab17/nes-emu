use super::isa::AddressingMode;
use super::utils::is_negative;
use crate::cpu::cpu::Cpu;
use crate::error::Result;
use crate::mem::utils::make_address;
use crate::mem::device::MemoryDevice;

pub fn effective_addr(am: AddressingMode, cpu: &mut Cpu) -> Result<u16> {
    use AddressingMode::*;
    match am {
        Absolute(addr) => Ok(addr),
        AbsoluteX(addr) => Ok(addr + cpu.reg.x as u16),
        AbsoluteY(addr) => Ok(addr.wrapping_add(cpu.reg.y as u16)),
        XIndirect(offset) => {
            let lo = cpu.bus.read((offset.wrapping_add(cpu.reg.x) & 0xFF) as u16)?;
            let hi = cpu.bus.read((offset.wrapping_add(cpu.reg.x).wrapping_add(1) & 0xFF) as u16)?;
            Ok(make_address(lo, hi))
        }
        IndirectY(offset) => {
            let lo = cpu.bus.read(offset as u16)?;
            let hi = cpu.bus.read((offset.wrapping_add(1)) as u16)?;
            Ok(make_address(lo, hi).wrapping_add(cpu.reg.y as u16))
        }
        ZeroPage(offset) => Ok(offset as u16),
        ZeroPageX(offset) => Ok(((offset as u16) + (cpu.reg.x as u16)) & 0xFF),
        ZeroPageY(offset) => Ok(((offset as u16) + (cpu.reg.y as u16)) & 0xFF),
        Immediate(_) | Relative(_) | Accumulator | Implied | Indirect(_) => {
            panic!("{:?} has no effective address.", am)
        }
    }
}

pub fn deref_byte(am: AddressingMode, cpu: &mut Cpu) -> Result<u8> {
    use AddressingMode::*;
    match am {
        Accumulator => Ok(cpu.reg.a),
        Immediate(val) => Ok(val),
        Implied | Indirect(_) | Relative(_) => {
            panic!("{:?} cannot be derefenced into a byte value.", am)
        }
        _ => {
            let addr = effective_addr(am, cpu)?;
            cpu.bus.read(addr)
        },
    }
}

pub fn deref_address(am: AddressingMode, cpu: &mut Cpu) -> Result<u16> {
    use AddressingMode::*;
    match am {
        Indirect(addr) => {
            if addr & 0xFF == 0xFF {
                // Simulate hardware bug when crossing pages
                let lo = cpu.bus.read(addr)?;
                let hi = cpu.bus.read(addr & 0xFF00)?;
                Ok(make_address(lo, hi))
            } else {
                let lo = cpu.bus.read(addr)?;
                let hi = cpu.bus.read(addr + 1)?;
                Ok(make_address(lo, hi))
            }
        }
        Absolute(addr) => Ok(addr),
        Relative(offset) => {
            if is_negative(offset) {
                Ok(cpu.reg.pc - (offset.wrapping_neg() as u16))
            } else {
                Ok(cpu.reg.pc + (offset as u16))
            }
        }
        _ => panic!("deref_address on {:?}", am),
    }
}

#[cfg(test)]
mod addressing_mode_tests {
    use super::AddressingMode::*;
    use crate::cpu::cpu::Cpu;
    use crate::mem::device::MemoryDevice;

    use super::deref_address;
    use super::deref_byte;

    // Some test cases come from examples in this post:
    // http://www.emulator101.com/6502-addressing-modes.html

    #[test]
    fn accumulator() {
        let mut cpu = Cpu::mock(None);
        cpu.reg.a = 42;
        assert_eq!(deref_byte(Accumulator, &mut cpu).unwrap(), 42);
    }

    #[test]
    fn absolute_byte() {
        let mut cpu = Cpu::mock(Some(&[0xFE, 0xCA, 0xEF, 0xBE]));
        assert_eq!(deref_byte(Absolute(0x0000), &mut cpu).unwrap(), 0xFE);
        assert_eq!(deref_byte(Absolute(0x0002), &mut cpu).unwrap(), 0xEF);
    }

    #[test]
    fn absolute_x_index() {
        let mut cpu = Cpu::mock(None);
        // no carry
        cpu.bus.write(0x08F7, 0x42).unwrap();
        cpu.reg.x = 0x7;
        assert_eq!(deref_byte(AbsoluteX(0x08F0), &mut cpu).unwrap(), 0x42);

        // carry
        cpu.bus.write(0x0200, 0x17).unwrap();
        cpu.reg.x = 0xFF;
        assert_eq!(deref_byte(AbsoluteX(0x101), &mut cpu).unwrap(), 0x17);
    }

    #[test]
    fn absolute_y_index() {
        let mut cpu = Cpu::mock(None);
        // no carry
        cpu.bus.write(0x08F7, 0x42).unwrap();
        cpu.reg.y = 0x7;
        assert_eq!(deref_byte(AbsoluteY(0x08F0), &mut cpu).unwrap(), 0x42);

        // carry
        cpu.bus.write(0x0200, 0x17).unwrap();
        cpu.reg.y = 0xFF;
        assert_eq!(deref_byte(AbsoluteY(0x101), &mut cpu).unwrap(), 0x17);
    }

    #[test]
    fn immediate() {
        let mut cpu = Cpu::mock(None);
        assert_eq!(deref_byte(Immediate(0x25), &mut cpu).unwrap(), 0x25);
        assert_eq!(deref_byte(Immediate(0xFF), &mut cpu).unwrap(), 0xFF);
        assert_eq!(deref_byte(Immediate(0x00), &mut cpu).unwrap(), 0x00);
    }

    #[test]
    fn x_indirect() {
        let mut cpu = Cpu::mock(None);
        cpu.reg.x = 0x04;
        cpu.bus.write(0x0024, 0x74).unwrap();
        cpu.bus.write(0x0025, 0x01).unwrap();
        cpu.bus.write(0x0174, 0x42).unwrap();
        assert_eq!(deref_byte(XIndirect(0x20), &mut cpu).unwrap(), 0x42);
    }

    #[test]
    fn indirect_y() {
        let mut cpu = Cpu::mock(None);
        cpu.reg.y = 0x10;
        cpu.bus.write(0x74, 0x28).unwrap();
        cpu.bus.write(0x75, 0x01).unwrap();
        cpu.bus.write(0x0138, 0x42).unwrap();
        assert_eq!(deref_byte(IndirectY(0x74), &mut cpu).unwrap(), 0x42);
    }

    #[test]
    fn zero_page() {
        let mut cpu = Cpu::mock(None);
        cpu.bus.write(0x00, 0xAB).unwrap();
        cpu.bus.write(0x22, 0xCD).unwrap();
        cpu.bus.write(0x33, 0xEF).unwrap();
        cpu.bus.write(0xFF, 0x77).unwrap();
        assert_eq!(deref_byte(ZeroPage(0x00), &mut cpu).unwrap(), 0xAB);
        assert_eq!(deref_byte(ZeroPage(0x22), &mut cpu).unwrap(), 0xCD);
        assert_eq!(deref_byte(ZeroPage(0x33), &mut cpu).unwrap(), 0xEF);
        assert_eq!(deref_byte(ZeroPage(0xFF), &mut cpu).unwrap(), 0x77);
    }

    #[test]
    fn zero_page_x() {
        let mut cpu = Cpu::mock(None);
        cpu.bus.write(0x20, 0x42).unwrap();
        cpu.reg.x = 0x60;

        // carry ignored
        // 0x60 + 0xC0 = 0x120 -> 0x20
        assert_eq!(deref_byte(ZeroPageX(0xC0), &mut cpu).unwrap(), 0x42);

        // no carry
        cpu.reg.x = 0x01;
        assert_eq!(deref_byte(ZeroPageX(0x1F), &mut cpu).unwrap(), 0x42);
    }

    #[test]
    fn zero_page_y() {
        let mut cpu = Cpu::mock(None);
        cpu.bus.write(0x20, 0x42).unwrap();
        cpu.reg.y = 0x60;

        // carry ignored
        // 0x60 + 0xC0 = 0x120 -> 0x20
        assert_eq!(deref_byte(ZeroPageY(0xC0), &mut cpu).unwrap(), 0x42);

        // no carry
        cpu.reg.y = 0x01;
        assert_eq!(deref_byte(ZeroPageY(0x1F), &mut cpu).unwrap(), 0x42);
    }

    #[test]
    fn relative() {
        let mut cpu = Cpu::mock(None);

        cpu.reg.pc = 0x100;
        // Positive offsets
        assert_eq!(deref_address(Relative(0x8), &mut cpu).unwrap(), 0x108);
        assert_eq!(deref_address(Relative(0x7F), &mut cpu).unwrap(), 0x17F);

        // Negative offsets
        assert_eq!(deref_address(Relative(0xFF), &mut cpu).unwrap(), 0x0FF);
        assert_eq!(deref_address(Relative(0x9C), &mut cpu).unwrap(), 0x9C);

        cpu.reg.pc = 0x0123;
        // 0x0123 - 0x3 = 0x1230
        assert_eq!(deref_address(Relative(0xFD), &mut cpu).unwrap(), 0x120);
        // 0x0123 - 0x5 = 0x11E
        assert_eq!(deref_address(Relative(0xFB), &mut cpu).unwrap(), 0x11E);
        // 0x0123 - 0x28 = 0xFB
        assert_eq!(deref_address(Relative(0xD8), &mut cpu).unwrap(), 0xFB);
        // 0x0123 - 0x80 = 0xA3
        assert_eq!(deref_address(Relative(0x80), &mut cpu).unwrap(), 0xA3);
    }

    #[test]
    fn absolute_addr() {
        let mut cpu = Cpu::mock(None);
        assert_eq!(deref_address(Absolute(0xCAFE), &mut cpu).unwrap(), 0xCAFE);
        assert_eq!(deref_address(Absolute(0xBEA7), &mut cpu).unwrap(), 0xBEA7);
        assert_eq!(deref_address(Absolute(0x0101), &mut cpu).unwrap(), 0x0101);
    }

    #[test]
    fn indirect() {
        let mut cpu = Cpu::mock(None);
        cpu.bus.write(0x100, 0x52).unwrap();
        cpu.bus.write(0x101, 0x3a).unwrap();
        assert_eq!(deref_address(Indirect(0x100), &mut cpu).unwrap(), 0x3a52);

        cpu.bus.write(0xFF, 0x76).unwrap();
        cpu.bus.write(0x100, 0x17).unwrap();
        assert_eq!(deref_address(Indirect(0xFF), &mut cpu).unwrap(), 0x1776);
    }
}
