use crate::cpu::isa::AddressingMode;

use super::{
    cpu::Cpu,
    isa::{Instr, Opcode},
};

#[derive(Clone, Copy)]
enum Mode {
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

// Makes lookup table shorter
// (Can't have dirty garbage)
type Op = Opcode;
type M = Mode;
type Tup = (Opcode, Mode, u8);
const INV: Tup = (Op::INVALID, M::Imp, 0);

#[rustfmt::skip]
const LOOKUP: [[Tup; 16]; 2] = [
    [(Op::BRK, M::Imp, 7), (Op::ORA, M::XInd, 6), INV, INV, (Op::NOP, M::Zpg, 3), (Op::ORA, M::Zpg, 3), (Op::ASL, M::Zpg, 5), INV, (Op::PHP, M::Imp, 3), (Op::ORA, M::Imm, 2), (Op::ASL, M::Imp, 2), INV, INV, (Op::ORA, M::Abs, 4), (Op::ASL, M::Abs, 6), INV],
    [(Op::BPL, M::Rel, 2), (Op::ORA, M::IndY, 5), INV, INV, (Op::NOP, M::ZpgX, 4), (Op::ORA, M::ZpgX, 4), (Op::ASL, M::ZpgX, 6), INV, (Op::CLC, M::Imp, 2), (Op::ORA, M::AbsY, 4), (Op::NOP, M::Imp, 2), INV, (Op::NOP, M::AbsX, 4), (Op::ORA, M::AbsX, 4), (Op::ASL, M::AbsX, 7), INV]
];

// Reads byte at current PC, then advances PC
fn read_next_byte(cpu: &mut Cpu) -> Option<u8> {
    let byte = cpu.bus.read(cpu.reg.pc);
    cpu.reg.pc += 1;
    byte
}

// Reads the 16-bit low endian value after the PC and converts it from low endian back to "correct" endian ;)
fn read_little_endian_u16(cpu: &mut Cpu) -> Option<u16> {
    let lo = read_next_byte(cpu);
    let hi = read_next_byte(cpu);
    lo.zip(hi).map(|(l, h)| ((h as u16) << 8) | (l as u16))
}

// TODO: Change to Result<> for better errors?
pub fn decode_instr(cpu: &mut Cpu) -> Option<Instr> {
    if let Some(next_instr) = read_next_byte(cpu) {
        let row = (next_instr & 0xF0) >> 4;
        let col = next_instr & 0xF;

        let (opcode, mode, n_cycles) = LOOKUP[row as usize][col as usize];
        if opcode == Op::INVALID {
            return None;
        }

        // functors an dat
        type M = Mode;
        type AM = AddressingMode;
        let address_mode = match mode {
            M::Acc => Some(AM::Accumulator),
            M::Abs => read_little_endian_u16(cpu).map(|addr| AM::Absolute(addr)),
            M::AbsX => read_little_endian_u16(cpu).map(|addr| AM::AbsoluteX(addr)),
            M::AbsY => read_little_endian_u16(cpu).map(|addr| AM::AbsoluteY(addr)),
            M::Imm => read_next_byte(cpu).map(|b| AM::Immediate(b)),
            M::Imp => Some(AM::Implied),
            M::Ind => read_little_endian_u16(cpu).map(|addr| AM::Indirect(addr)),
            M::XInd => read_next_byte(cpu).map(|b| AM::XIndirect(b)),
            M::IndY => read_next_byte(cpu).map(|b| AM::IndrectY(b)),
            M::Rel => read_next_byte(cpu).map(|b| AM::Relative(b)),
            M::Zpg => read_next_byte(cpu).map(|b| AM::ZeroPage(b)),
            M::ZpgX => read_next_byte(cpu).map(|b| AM::ZeroPageX(b)),
            M::ZpgY => read_next_byte(cpu).map(|b| AM::ZeroPageY(b)),
        };

        address_mode.map(|am| Instr {
            op: opcode,
            mode: am,
            num_cycles: n_cycles,
        })
    } else {
        None
    }
}

#[cfg(test)]
mod decode_tests {
    use crate::cpu::{
        cpu::Cpu,
        isa::{AddressingMode, Instr, Opcode},
    };

    use super::decode_instr;

    #[test]
    fn decode_tests() {
        let instrs = [
            0x1D, 0xEF, 0xBE, // ORA absX $BEEF
            0x0E, 0xFE, 0xCA, // ASL abs $CAFE
            0x15, 0xBB       // ORA zpgX $BB
        ];
        let mut cpu = Cpu::mock(&instrs);
        cpu.reg.pc = 0;

        assert_eq!(
            decode_instr(&mut cpu),
            Some(Instr {
                op: Opcode::ORA,
                mode: AddressingMode::AbsoluteX(0xBEEF),
                num_cycles: 4
            })
        );

        assert_eq!(
            decode_instr(&mut cpu),
            Some(Instr {
                op: Opcode::ASL,
                mode: AddressingMode::Absolute(0xCAFE),
                num_cycles: 6
            })
        );

        assert_eq!(
            decode_instr(&mut cpu),
            Some(Instr {
                op: Opcode::ORA,
                mode: AddressingMode::ZeroPageX(0xBB),
                num_cycles: 4
            })
        );
    }
}
