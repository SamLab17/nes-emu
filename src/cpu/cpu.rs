use super::decode::decode_instr;
use super::exec::exec_instr;
use super::reg::{Registers, StatusFlags};
use crate::mem::utils::make_address;
use crate::mem::{bus::MemoryBus, device::MemoryDevice, ram::Ram};
use crate::error::Result;

pub const STACK_OFFSET: u16 = 0x1000;

#[derive(Debug, Clone, Copy)]
pub enum Interrupt {
    Request,
    Reset,
    NonMaskable
}

impl Interrupt {
    pub fn vector(&self) -> u16 {
        match self {
            Self::Request => 0xFFFA,
            Self::Reset => 0xFFFC,
            Self::NonMaskable => 0xFFFE
        }
    }
}

pub struct Cpu {
    pub reg: Registers,
    pub bus: Box<dyn MemoryDevice>,
    pub interrupt: Option<Interrupt>
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            reg: Default::default(),
            bus: Box::new(MemoryBus::new()),
            interrupt: None,
        }
    }

    pub fn mock(mem: &[u8]) -> Self {
        Cpu {
            reg: Default::default(),
            bus: Box::new(Ram::from(mem)),
            interrupt: None,
        }
    }

    pub fn run_next_instr(&mut self) -> Result<u8> {
        // Check for interrupts
        if let Some(interrupt) = self.interrupt {
            let interrupts_enabled = !self.reg.status.contains(StatusFlags::INTERRUPT_DISABLE);
            if matches!(interrupt, Interrupt::NonMaskable) || interrupts_enabled {
                let vector = interrupt.vector();
                let isr_addr = make_address(self.bus.read(vector)?, self.bus.read(vector+1)?);
                // Jump to Interrupt Handler 
                self.reg.pc = isr_addr;
            }
        }
        
        // Decode and run the next instruction
        exec_instr(decode_instr(self)?, self)
    }
}