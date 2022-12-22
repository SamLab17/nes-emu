use super::decode::decode_instr;
use super::exec::exec_instr;
use super::reg::{Registers, StatusFlags};
use crate::cart::cart::Cartridge;
use crate::cart::mock::mock_cart;
use crate::error::Result;
use crate::mem::bus::MemoryBusBuilder;
use crate::mem::utils::make_address;
use crate::mem::{bus::MemoryBus, device::MemoryDevice};
use crate::ppu::ppu::Ppu;

pub const STACK_OFFSET: u16 = 0x1000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Interrupt {
    Request,
    Reset,
    NonMaskable,
}

impl Interrupt {
    pub fn vector(&self) -> u16 {
        match self {
            Self::Request => 0xFFFA,
            Self::Reset => 0xFFFC,
            Self::NonMaskable => 0xFFFE,
        }
    }
}

pub struct Cpu {
    pub reg: Registers,
    pub bus: MemoryBus,
    pub interrupt: Option<Interrupt>,
}

impl Cpu {
    pub fn new(cart: Cartridge) -> Self {
        Self {
            reg: Registers {
                pc: 0x34,
                sp: 0xFD,
                ..Registers::default()
            },
            bus: MemoryBusBuilder::new().with_cart(cart).build(),
            interrupt: None,
        }
    }

    // CPU with only RAM for unit tests
    pub fn mock(init_ram: Option<&[u8]>) -> Self {
        Cpu {
            reg: Default::default(),
            bus: MemoryBusBuilder::new().with_ram(init_ram).build(),
            interrupt: None,
        }
    }

    // Returns the number of cycles the instruction takes
    pub fn run_next_instr(&mut self) -> Result<u8> {
        // Check for interrupts
        if let Some(interrupt) = self.interrupt {
            let interrupts_enabled = !self.reg.status.contains(StatusFlags::INTERRUPT_DISABLE);
            if interrupt == Interrupt::NonMaskable || interrupts_enabled {
                let vector = interrupt.vector();
                let isr_addr = make_address(self.bus.read(vector)?, self.bus.read(vector + 1)?);
                // Jump to Interrupt Handler
                self.reg.pc = isr_addr;
            }
        }

        // Decode and run the next instruction
        exec_instr(decode_instr(self)?, self)
    }
}

/*
pub struct CpuBuilder {
    cart: Cartridge,
    ppu: Option<Ppu>,
}

impl CpuBuilder {
    pub fn new() -> Self {
        CpuBuilder {
            cart: mock_cart(),
            ppu: None,
        }
    }

    pub fn with_cart(mut self, cart: Cartridge) -> Self {
        self.cart = cart;
        self
    }

    pub fn with_ppu(mut self, ppu: Ppu) -> Self {
        self.ppu = Some(ppu);
        self
    }

    pub fn build(self) -> Cpu {
        Cpu {
            reg: Registers {
                pc: 0x34,
                sp: 0xFD,
                ..Registers::default()
            },
            bus: MemoryBusBuilder::new()
                .with_cart(self.cart)
                .with_ppu(self.ppu.unwrap_or_default())
                .build(),
            interrupt: None,
        }
    }
}
*/