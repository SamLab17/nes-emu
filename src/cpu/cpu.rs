use super::decode::decode_instr;
use super::exec::exec_instr;
use super::reg::Registers;
use crate::mem::{bus::MemoryBus, device::MemoryDevice, ram::Ram};
use crate::error::Result;

pub struct Cpu {
    pub reg: Registers,
    pub bus: Box<dyn MemoryDevice>,
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            reg: Default::default(),
            bus: Box::new(MemoryBus::new())
        }
    }

    pub fn mock(mem: &[u8]) -> Self {
        Cpu {
            reg: Default::default(),
            bus: Box::new(Ram::from(mem))
        }
    }

    pub fn run_next_instr(&mut self) -> Result<u8> {
        let i = decode_instr(self)?;
        exec_instr(i, self)
    }
}