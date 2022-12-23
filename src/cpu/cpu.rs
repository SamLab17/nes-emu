use super::decode::fetch_instr;
use super::exec::exec_instr;
use super::isa::Instr;
use super::reg::{Registers, StatusFlags};
use crate::cart::cart::Cartridge;
use crate::cart::mock::mock_cart;
use crate::error::Result;
use crate::mem::bus::MemoryBusBuilder;
use crate::mem::utils::make_address;
use crate::mem::{bus::MemoryBus, device::MemoryDevice};
use crate::ppu::ppu::{Ppu, Frame};

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

const NUM_TICKS_PER_CPU_CYCLE: u8 = 3;

pub struct Cpu {
    pub reg: Registers,
    pub bus: MemoryBus,
    pub interrupt: Option<Interrupt>,
    cycles_left: u16,
    ticks_left : u8,
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
            interrupt: Some(Interrupt::Reset),
            cycles_left: 0,
            ticks_left: 0,
        }
    }

    // CPU with only RAM for unit tests
    pub fn mock(init_ram: Option<&[u8]>) -> Self {
        Cpu {
            reg: Default::default(),
            bus: MemoryBusBuilder::new().with_ram(init_ram).build(),
            interrupt: None,
            cycles_left: 0,
            ticks_left: 0,
        }
    }

    pub fn reset(&mut self) -> Result<()> {
        self.reg.sp -= 3;
        self.reg.status.insert(StatusFlags::INTERRUPT_DISABLE);
        self.interrupt = Some(Interrupt::Reset);
        self.bus.ppu.reset()?;
        Ok(())
    }

    // Returns the number of cycles the instruction takes
    fn run_next_instr(&mut self) -> Result<u16> {
        // Check for interrupts
        if let Some(interrupt) = self.interrupt {
            let interrupts_enabled = !self.reg.status.contains(StatusFlags::INTERRUPT_DISABLE);
            if interrupt == Interrupt::NonMaskable || interrupts_enabled {
                let vector = interrupt.vector();
                let isr_addr = make_address(self.bus.read(vector)?, self.bus.read(vector + 1)?);
                // Jump to Interrupt Handler
                self.reg.pc = isr_addr;
                // Clear interrupt
                self.interrupt = None;
            }
        }

        let pc = self.reg.pc;
        // Decode and run the next instruction
        let i = fetch_instr(self)?;
        // println!("0x{:X}: {}", pc, i);
        exec_instr(i, self)
    }

    fn cycle(&mut self) -> Result<()> {
        if self.cycles_left == 0 {
            self.cycles_left = self.run_next_instr()?;
        } else {
            self.cycles_left -= 1;
        }
        Ok(())
    }

    pub fn system_tick(&mut self) -> Result<Option<Frame>> {
        let frame = self.bus.ppu_tick()?;

        if self.ticks_left == 0 {
            self.cycle()?;
            self.ticks_left = NUM_TICKS_PER_CPU_CYCLE;
        } else {
            self.ticks_left -= 1;
        }

        Ok(frame)
    }

    pub fn next_frame(&mut self) -> Result<Frame> {
        let mut num_ticks = 0;
        loop {
            num_ticks += 1;
            if let Some(frame) = self.system_tick()? {
                println!("num ticks to render: {num_ticks}");
                return Ok(frame);
            }
        }
    }

    pub fn debug_frame(&self) -> Frame {
        self.bus.ppu.buffer
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