use super::reg::Registers;
use crate::mem::bus::MemoryBus;

pub struct Cpu {
    pub reg: Registers,
    pub bus: MemoryBus,
}

impl Cpu {
    fn new() -> Self {
        Cpu {
            reg: Default::default(),
            bus: MemoryBus::new()
        }
    }
}