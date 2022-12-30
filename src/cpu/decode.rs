use crate::error::Result;
use crate::{cpu::isa::AddressingMode, mem::utils::make_address};
use std::{error::Error, fmt};

#[derive(Debug, Clone)]
enum DecodeError {
    InvalidOpcode(u8, u16),
}
impl Error for DecodeError {}
impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InvalidOpcode(bb, addr) => write!(
                f,
                "Attempted to decode invalid opcode: {:X} at addr {:X}",
                *bb, addr
            ),
        }
    }
}

use super::{
    cpu::Cpu,
    isa::{Instr, Opcode},
};

// Makes lookup table shorter
// (Can't have dirty garbage)
pub mod instr_lookup {
    #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
    pub enum Mode {
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
        ZpgY,
    }

    use crate::cpu::isa::Opcode::{self, *};
    use Mode::*;
    type InstrTup = (Opcode, Mode, u8);
    const INV: InstrTup = (INVALID, Imp, 0);

    #[rustfmt::skip]
    pub const LOOKUP: [[InstrTup; 16]; 16] = [
        [(BRK, Imp, 7), (ORA, XInd, 6), INV, INV, (NOP, Zpg, 3),  (ORA, Zpg, 3),  (ASL, Zpg, 5),  INV, (PHP, Imp, 3), (ORA, Imm, 2),  (ASL, Acc, 2), INV, (NOP, Abs, 4), (ORA, Abs, 4), (ASL, Abs, 6), INV],
        [(BPL, Rel, 2), (ORA, IndY, 5), INV, INV, (NOP, ZpgX, 4), (ORA, ZpgX, 4), (ASL, ZpgX, 6), INV, (CLC, Imp, 2), (ORA, AbsY, 4), (NOP, Imp, 2), INV, (NOP, AbsX, 4), (ORA, AbsX, 4), (ASL, AbsX, 7), INV],
        [(JSR, Abs, 6), (AND, XInd, 6), INV, INV, (BIT, Zpg, 3),  (AND, Zpg, 3),  (ROL, Zpg, 5),  INV, (PLP, Imp, 4), (AND, Imm, 2),  (ROL, Acc, 2), INV, (BIT, Abs, 4), (AND, Abs, 4), (ROL, Abs, 6), INV],
        [(BMI, Rel, 2), (AND, IndY, 5), INV, INV, (NOP, ZpgX, 4), (AND, ZpgX, 4), (ROL, ZpgX, 6), INV, (SEC, Imp, 2), (AND, AbsY, 4), (NOP, Imp, 2), INV, (NOP, AbsX, 4), (AND, AbsX, 4), (ROL, AbsX, 7), INV],
        [(RTI, Imp, 6), (EOR, XInd, 6), INV, INV, (NOP, Zpg, 3),  (EOR, Zpg, 3),  (LSR, Zpg, 5),  INV, (PHA, Imp, 3), (EOR, Imm, 2),  (LSR, Acc, 2), INV, (JMP, Abs, 3), (EOR, Abs, 4), (LSR, Abs, 6), INV],
        [(BVC, Rel, 2), (EOR, IndY, 5), INV, INV, (NOP, ZpgX, 4), (EOR, ZpgX, 4), (LSR, ZpgX, 6), INV, (CLI, Imp, 2), (EOR, AbsY, 4), (NOP, Imp, 2), INV, (NOP, AbsX, 4), (EOR, AbsX, 4), (LSR, AbsX, 7), INV],
        [(RTS, Imp, 6), (ADC, XInd, 6), INV, INV, (NOP, Zpg, 3),  (ADC, Zpg, 3),  (ROR, Zpg, 5),  INV, (PLA, Imp, 4), (ADC, Imm, 2),  (ROR, Acc, 2), INV, (JMP, Ind, 5), (ADC, Abs, 4), (ROR, Abs, 6), INV],
        [(BVS, Rel, 2), (ADC, IndY, 5), INV, INV, (NOP, ZpgX, 4), (ADC, ZpgX, 4), (ROR, ZpgX, 6), INV, (SEI, Imp, 2), (ADC, AbsY, 4), (NOP, Imp, 2), INV, (NOP, AbsX, 4), (ADC, AbsX, 4), (ROR, AbsX, 7), INV],
        [(NOP, Imm, 2), (STA, XInd, 6), INV, INV, (STY, Zpg, 3),  (STA, Zpg, 3),  (STX, Zpg, 3),  INV, (DEY, Imp, 2), (NOP, Imm, 2),  (TXA, Imp, 2), INV, (STY, Abs, 4), (STA, Abs, 4), (STX, Abs, 4), INV],
        [(BCC, Rel, 2), (STA, IndY, 6), INV, INV, (STY, ZpgX, 4), (STA, ZpgX, 4), (STX, ZpgY, 4), INV, (TYA, Imp, 2), (STA, AbsY, 5), (TXS, Imp, 2), INV, (NOP, AbsX, 4), (STA, AbsX, 5), INV, INV],
        [(LDY, Imm, 2), (LDA, XInd, 6), (LDX, Imm, 2), (LAX, XInd, 6), (LDY, Zpg, 3), (LDA, Zpg, 3), (LDX, Zpg, 3), (LAX, Zpg, 3), (TAY, Imp, 2), (LDA, Imm, 2), (TAX, Imp, 2), (LAX, Imm, 3), (LDY, Abs, 4), (LDA, Abs, 4), (LDX, Abs, 4), (LAX, Abs, 4)],
        [(BCS, Rel, 2), (LDA, IndY, 5), INV, (LAX, IndY, 5), (LDY, ZpgX, 4), (LDA, ZpgX, 4), (LDX, ZpgY, 4), (LAX, ZpgY, 4), (CLV, Imp, 2), (LDA, AbsY, 4), (TSX, Imp, 2), INV, (LDY, AbsX, 4), (LDA, AbsX, 4), (LDX, AbsY, 4), (LAX, AbsY, 4)],
        [(CPY, Imm, 2), (CMP, XInd, 6), INV, INV, (CPY, Zpg, 3),  (CMP, Zpg, 3),  (DEC, Zpg, 5),  INV, (INY, Imp, 2), (CMP, Imm, 2),  (DEX, Imp, 2), INV, (CPY, Abs, 4), (CMP, Abs, 4), (DEC, Abs, 6), INV],
        [(BNE, Rel, 2), (CMP, IndY, 5), INV, INV, (NOP, ZpgX, 4), (CMP, ZpgX, 4), (DEC, ZpgX, 6), INV, (CLD, Imp, 2), (CMP, AbsY, 4), (NOP, Imp, 2), INV, (NOP, AbsX, 4), (CMP, AbsX, 4), (DEC, AbsX, 7), INV],
        [(CPX, Imm, 2), (SBC, XInd, 6), INV, INV, (CPX, Zpg, 3),  (SBC, Zpg, 3),  (INC, Zpg, 5),  INV, (INX, Imp, 2), (SBC, Imm, 2),  (NOP, Imp, 2), INV, (CPX, Abs, 4), (SBC, Abs, 4), (INC, Abs, 6), INV],
        [(BEQ, Rel, 2), (SBC, IndY, 5), INV, INV, (NOP, ZpgX, 4), (SBC, ZpgX, 4), (INC, ZpgX, 6), INV, (SED, Imp, 2), (SBC, AbsY, 4), (NOP, Imp, 2), INV, (NOP, AbsX, 4), (SBC, AbsX, 4), (INC, AbsX, 7), INV],
    ];
}

// Reads byte at current PC, then advances PC
fn read_next_byte(cpu: &mut Cpu) -> Result<u8> {
    let byte = cpu.read(cpu.reg.pc)?;
    cpu.reg.pc += 1;
    Ok(byte)
}

// Reads the 16-bit low endian value after the PC and converts
// it from low endian back to the objectively-better-endian ;)
fn read_little_endian_u16(cpu: &mut Cpu) -> Result<u16> {
    let lo = read_next_byte(cpu)?;
    let hi = read_next_byte(cpu)?;
    Ok(make_address(lo, hi))
}

pub fn fetch_instr(cpu: &mut Cpu) -> Result<(Instr, u16)> {
    let next_instr = read_next_byte(cpu)?;
    let row = (next_instr & 0xF0) >> 4;
    let col = next_instr & 0xF;

    let (opcode, mode, ncycles) = instr_lookup::LOOKUP[row as usize][col as usize];
    if opcode == Opcode::INVALID {
        return Err(Box::new(DecodeError::InvalidOpcode(
            next_instr,
            cpu.reg.pc - 1,
        )));
    }

    // functors an dat
    type M = instr_lookup::Mode;
    type AM = AddressingMode;
    let address_mode = match mode {
        M::Acc => Ok(AM::Accumulator),
        M::Abs => read_little_endian_u16(cpu).map(|addr| AM::Absolute(addr)),
        M::AbsX => read_little_endian_u16(cpu).map(|addr| AM::AbsoluteX(addr)),
        M::AbsY => read_little_endian_u16(cpu).map(|addr| AM::AbsoluteY(addr)),
        M::Imm => read_next_byte(cpu).map(|b| AM::Immediate(b)),
        M::Imp => Ok(AM::Implied),
        M::Ind => read_little_endian_u16(cpu).map(|addr| AM::Indirect(addr)),
        M::XInd => read_next_byte(cpu).map(|b| AM::XIndirect(b)),
        M::IndY => read_next_byte(cpu).map(|b| AM::IndirectY(b)),
        M::Rel => read_next_byte(cpu).map(|b| AM::Relative(b)),
        M::Zpg => read_next_byte(cpu).map(|b| AM::ZeroPage(b)),
        M::ZpgX => read_next_byte(cpu).map(|b| AM::ZeroPageX(b)),
        M::ZpgY => read_next_byte(cpu).map(|b| AM::ZeroPageY(b)),
    };

    address_mode.map(|am| {
        (
            Instr {
                op: opcode,
                mode: am,
            },
            ncycles as u16,
        )
    })
}

#[cfg(test)]
mod decode_tests {
    use crate::cpu::{
        cpu::Cpu,
        isa::{AddressingMode, Instr, Opcode},
    };

    use super::fetch_instr;

    // Checks proper decode of instruction (get correct instruction, read correct # of bytes) and
    // and checks the # of cycles for the instruction
    fn check_decode(binary: &[u8], op: Opcode, mode: AddressingMode, n_cycles: u16) {
        let mut cpu = Cpu::mock(Some(binary));
        cpu.reg.pc = 0;
        let instr = Instr { op, mode };

        let (act_instr, act_n_cycles) = fetch_instr(&mut cpu).unwrap();
        assert_eq!(act_instr, instr);
        assert_eq!(binary.len(), cpu.reg.pc as usize);
        assert_eq!(act_n_cycles, n_cycles);
    }

    #[test]
    fn decode_test() {
        use AddressingMode::*;
        use Opcode::*;
        check_decode(&[0x1D, 0xEF, 0xBE], ORA, AbsoluteX(0xBEEF), 4);
        check_decode(&[0x0E, 0xFE, 0xCA], ASL, Absolute(0xCAFE), 6);
        check_decode(&[0x15, 0xBB], ORA, ZeroPageX(0xBB), 4);
        check_decode(&[0x08], PHP, Implied, 3);
    }
}
