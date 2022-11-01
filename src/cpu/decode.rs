use crate::{mem::device::MemoryDevice, cpu::isa::AddressingMode};

use super::{isa::{Instr, Opcode}, cpu::Cpu};

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
    ZpgY
}

type Op = Opcode;
type M = Mode;
type Tup = (Opcode, Mode, u8);

const INV: Tup = (Op::INVALID, M::Imp, 0);

// Tuples (Opcode, Mode, # cycles)
const LOOKUP: [[Tup; 16]; 2] = [
    [(Op::BRK, M::Imp, 7), (Op::ORA, M::XInd, 6), INV, INV, (Op::NOP, M::Zpg, 3), (Op::ORA, M::Zpg, 3), (Op::ASL, M::Zpg, 5), INV, (Op::PHP, M::Imp, 3), (Op::ORA, M::Imm, 2), (Op::ASL, M::Imp, 2), INV, INV, (Op::ORA, M::Abs, 4), (Op::ASL, M::Abs, 4), INV],
    [(Op::BPL, M::Rel, 2), (Op::ORA, M::IndY, 5), INV, INV, (Op::NOP, M::ZpgX, 4), (Op::ORA, M::ZpgX, 4), (Op::ASL, M::ZpgX, 6), INV, (Op::CLC, M::Imp, 2), (Op::ORA, M::AbsY, 4), (Op::NOP, M::Imp, 2), INV, (Op::NOP, M::AbsX, 4), (Op::ORA, M::AbsX, 4), (Op::ASL, M::AbsX, 7), INV]
];

fn little_endian_addr(lo: u8, hi: u8) -> u16 {
    ((hi as u16) << 8) | (lo as u16)
}

pub fn decode_instr(cpu: &mut Cpu) -> Option<Instr> {
    // Reads byte at current PC, then advances PC
    let read_next_byte = || {
        let byte = cpu.bus.read(cpu.reg.pc);
        cpu.reg.pc += 1;
        byte
    };

    if let Some(next_instr) = read_next_byte() {
        let row = next_instr & 0xF0;
        let col = next_instr & 0xF;
        let (opcode, mode, n_cycles) = LOOKUP[row as usize][col as usize];
        if opcode == Op::INVALID {
            return None
        }

        // Advance pc past opcode
        cpu.reg.pc += 1;
        type AM = AddressingMode;

        let read_little_endian_addr = || {
            let lo = read_next_byte();
            let hi = read_next_byte();
            if let Some((l, h)) = lo.zip(hi) {
                Some(little_endian_addr(l, h))
            } else { 
                None 
            }
        };

        let address_mode =  match mode {
            Acc => Some(AM::Accumulator),
            Abs => read_little_endian_addr().map(|addr| AM::Absolute(addr)),
            AbsX => read_little_endian_addr().map(|addr| AM::AbsoluteX(addr)),
            AbsY => read_little_endian_addr().map(|addr| AM::AbsoluteY(addr)),
            Imm => read_next_byte().map(|b| AM::Immediate(b)),
            Imp => Some(AM::Implied),
            XInd => read_next_byte().map(|b| AM::XIndirect(b)),
            IndY => read_next_byte().map(|b| AM::IndrectY(b)),
            Rel => read_next_byte().map(|b| AM::Relative(b)),
            Zpg => read_next_byte().map(|b| AM::ZeroPage(b)),
            ZpgX => read_next_byte().map(|b| AM::ZeroPageX(b)),
            ZpgY => read_next_byte().map(|b| AM::ZeroPageY(b))
        };

        address_mode.map(|am| Instr {op:opcode, mode: am, num_cycles: n_cycles})
    } else {
        None
    }
}