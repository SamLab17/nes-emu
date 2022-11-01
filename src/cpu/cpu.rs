use super::reg::Registers;
use crate::mem::{bus::MemoryBus, device::MemoryDevice, ram::Ram};

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
}