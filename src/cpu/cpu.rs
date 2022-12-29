use super::decode::fetch_instr;
use super::exec::{exec_instr, push_stack, push_stack_addr};
use super::isa::Instr;
use super::reg::{Registers, StatusFlags};
use crate::cart::cart::Cartridge;
use crate::error::Result;
use crate::mem::bus::MemoryBus;
use crate::mem::bus::MemoryBusBuilder;
use crate::mem::utils::make_address;
use crate::ppu::ppu::Frame;

pub const STACK_OFFSET: u16 = 0x100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Interrupt {
    Request,
    Reset,
    NonMaskable,
}

impl Interrupt {
    pub fn vector(&self) -> u16 {
        match self {
            Self::NonMaskable => 0xFFFA,
            Self::Reset => 0xFFFC,
            Self::Request => 0xFFFE,
        }
    }
    pub fn num_cycles(&self) -> u16 {
        match self {
            Self::NonMaskable | Self::Reset => 8,
            Self::Request => 7,
        }
    }
}

const NUM_TICKS_PER_CPU_CYCLE: u8 = 3;

pub struct Cpu {
    pub reg: Registers,
    pub bus: MemoryBus,
    pub interrupt: Option<Interrupt>,
    cycles_left: u16,
    ticks_left: u8,
    log_cycles: u64,
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
            log_cycles: 0,
        }
    }

    // CPU with only RAM for unit tests
    #[allow(dead_code)]
    pub fn mock(init_ram: Option<&[u8]>) -> Self {
        Cpu {
            reg: Default::default(),
            bus: MemoryBusBuilder::new().with_ram(init_ram).build(),
            interrupt: None,
            cycles_left: 0,
            ticks_left: 0,
            log_cycles: 0,
        }
    }

    pub fn reset(&mut self) -> Result<()> {
        let isr = Interrupt::Reset.vector();
        self.reg.pc = make_address(self.bus.read(isr)?, self.bus.read(isr + 1)?);

        self.reg.a = 0;
        self.reg.x = 0;
        self.reg.y = 0;
        self.reg.sp = 0xFD;
        self.reg.status = StatusFlags::from_bits(0).unwrap();
        self.reg.status.insert(StatusFlags::UNUSED);
        self.reg.status.insert(StatusFlags::INTERRUPT_DISABLE);
        self.cycles_left = 7;

        self.bus.ppu.reset()?;

        Ok(())
    }

    // Returns the number of cycles the instruction takes
    fn run_next_instr(&mut self, log: Option<&mut String>) -> Result<u16> {
        // Check for interrupts
        if let Some(interrupt) = self.interrupt {
            let interrupts_enabled = !self.reg.status.contains(StatusFlags::INTERRUPT_DISABLE);
            if interrupt != Interrupt::Request || interrupts_enabled {
                let vector = interrupt.vector();
                let isr_addr = make_address(self.bus.read(vector)?, self.bus.read(vector + 1)?);

                push_stack_addr(self, self.reg.pc)?;
                self.reg.status.remove(StatusFlags::BREAK);
                self.reg.status.insert(StatusFlags::UNUSED);
                self.reg.status.insert(StatusFlags::INTERRUPT_DISABLE);
                push_stack(self, self.reg.status.bits())?;

                // Jump to Interrupt Handler
                // self.reg.pc = make_address(self.bus.read(isr_addr)?, self.bus.read(isr_addr + 1)?);
                self.reg.pc = isr_addr;

                // Clear interrupt
                self.interrupt = None;

                return Ok(interrupt.num_cycles());
            }
        }
        // Address of the instruction we're about to fetch (fetch_instr modifies self.reg.pc)
        let pc = self.reg.pc;
        // Decode and run the next instruction
        let (i, ncycles) = fetch_instr(self)?;
        // print!("{:X} {:<40} ", pc, format!("{}", i));
        // println!("{}", self.reg);
        // print!(" {:3},{:3}", self.bus.ppu.scanline, self.bus.ppu.cycle);
        // println!(" CYC:{}", self.log_cycles);
        if let Some(log) = log {
            log.push_str(&format!(
                "{:04X} {:?} {} PPU:{:3},{:3} CYC:{}\n",
                pc, i.op, self.reg, self.bus.ppu.scanline, self.bus.ppu.cycle, self.log_cycles
            ));
        };

        match exec_instr(i, self) {
            Ok(extra_cycles) => Ok(ncycles + extra_cycles),
            Err(e) => {
                eprintln!("An error occurred trying to execute {i}");
                eprintln!("Program Counter: 0x{pc:04X}");
                eprintln!("Error: {:?}", e);
                eprintln!("CPU Registers: {}", self.reg);
                Err(e)
            }
        }
    }

    pub fn peek_next_instr(&mut self, offset: u16) -> Result<(u16, Instr)> {
        let restore_pc = self.reg.pc;
        let mut prev_pc = restore_pc;
        let mut i = 0;
        while let Ok((instr, _)) = fetch_instr(self) {
            if i >= offset {
                self.reg.pc = restore_pc;
                return Ok((prev_pc, instr));
            }
            prev_pc = self.reg.pc;
            i += 1;
        }
        self.reg.pc = restore_pc;
        Err("no valid instructions left".into())
    }

    fn cycle(&mut self, log: Option<&mut String>) -> Result<()> {
        // println!("cpu cycle");
        if self.cycles_left == 0 {
            self.cycles_left = self.run_next_instr(log)?;
        }

        self.cycles_left -= 1;
        self.log_cycles += 1;
        Ok(())
    }

    pub fn system_tick(&mut self, log: Option<&mut String>) -> Result<Option<Frame>> {
        if self.ticks_left == 0 {
            self.cycle(log)?;
            self.ticks_left = NUM_TICKS_PER_CPU_CYCLE;
        }
        self.ticks_left -= 1;

        let (frame, int) = self.bus.ppu.tick()?;
        self.interrupt = int;

        Ok(frame)
    }

    pub fn next_frame(&mut self) -> Result<Frame> {
        loop {
            if let Some(frame) = self.system_tick(None)? {
                return Ok(frame);
            }
        }
    }

    pub fn debug_frame(&self) -> Frame {
        self.bus.ppu.buffer.clone()
    }
}

#[cfg(test)]
mod cpu_test {
    use crate::{cart::builder::build_cartridge, cpu::cpu::Cpu, ines::parse::INesFile};

    static NESTEST: &'static [u8] = include_bytes!("../../roms/nestest.nes");
    static NESTEST_LOG: &'static str = include_str!("../../nestest-trimmed.log");

    #[test]
    fn nestest() {
        let rom = INesFile::try_from(&NESTEST.to_vec()).unwrap();
        let mut cpu = Cpu::new(build_cartridge(&rom).unwrap());
        cpu.reset().unwrap();
        cpu.reg.pc = 0xC000;

        const N_INSTR: usize = 5000;
        let mut log = String::new();
        let mut n = 0;
        while n < N_INSTR {
            if cpu.cycles_left == 0 && cpu.ticks_left == 0 {
                n += 1;
            }
            cpu.system_tick(Some(&mut log)).unwrap();
        }

        assert_eq!(
            log.trim_end(),
            NESTEST_LOG
                .lines()
                .take(N_INSTR)
                .collect::<Vec<&str>>()
                .join("\n")
        )
    }
}
