use sdl2::pixels::Color;

use super::decode::fetch_instr;
use super::exec::{exec_instr, push_stack, push_stack_addr};
use super::isa::Instr;
use super::reg::{Registers, StatusFlags};
use crate::cart::cart::Cartridge;
use crate::controller::ControllerRef;
use crate::error::Result;
use crate::mem::bus::MemoryBus;
use crate::mem::bus::MemoryBusBuilder;
use crate::mem::utils::make_address;
use crate::ppu::ppu::{Frame, PatternTable};

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
    bus: MemoryBus,
    pub interrupt: Option<Interrupt>,
    cycles_left: u16, // Cycles left before next instruction
    ticks_left: u8, // Ticks left before next CPU cycle
    num_cpu_cycles: u64, // Number of CPU cycles elapsed
    num_system_ticks: u64 // Number of system ticks elapsed
}

impl Cpu {
    pub fn new(cart: Cartridge, controller1: Option<ControllerRef>, controller2: Option<ControllerRef>) -> Self {
        Self {
            reg: Registers {
                pc: 0x34,
                sp: 0xFD,
                ..Registers::default()
            },
            bus: MemoryBusBuilder::new().with_cart(cart).with_controllers(controller1, controller2).build(),
            interrupt: Some(Interrupt::Reset),
            cycles_left: 0,
            ticks_left: 0,
            num_cpu_cycles: 0,
            num_system_ticks: 0
        }
    }

    // CPU with only RAM for unit tests
    #[allow(dead_code)]
    pub fn mock(init_ram: Option<&[u8]>) -> Self {
        Self {
            reg: Default::default(),
            bus: MemoryBusBuilder::new().with_ram(init_ram).build(),
            interrupt: None,
            cycles_left: 0,
            ticks_left: 0,
            num_cpu_cycles: 0,
            num_system_ticks: 0
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
       
        // Log instructions being run
        if let Some(log) = log {
            log.push_str(&format!(
                "{:04X} {:?} {} PPU:{:3},{:3} CYC:{}\n",
                pc, i.op, self.reg, self.bus.ppu.scanline, self.bus.ppu.cycle, self.num_cpu_cycles
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
        if self.cycles_left == 0 {
            self.cycles_left = self.run_next_instr(log)?;
        }

        self.cycles_left -= 1;
        self.num_cpu_cycles += 1;
        Ok(())
    }

    pub fn system_tick(&mut self, log: Option<&mut String>) -> Result<Option<Frame>> {
        self.num_system_ticks += 1;

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

    pub fn read(&mut self, addr: u16) -> Result<u8> {
        self.bus.read(addr)
    }

    pub fn write(&mut self, addr: u16, byte: u8) -> Result<()> {
        if addr == 0x4014 {
            // Intercept this write, PPU won't see it
            // Do the entire DMA
            for off in 0..=0xFFu8 {
                let addr = make_address(off, byte);
                let data = self.read(addr)?;
                self.bus.ppu.oam_write(off, data);
            }

            self.cycles_left = 512;
            if self.num_cpu_cycles % 2 == 1 {
                self.cycles_left += 1
            }

            Ok(())
        } else {
            self.bus.write(addr, byte)
        }
    }

    pub fn debug_pattern_tables(&mut self, palette: u8, bg: bool) -> Result<(PatternTable, PatternTable)> {
        self.bus.ppu.debug_pattern_tables(palette, bg)
    }

    pub fn debug_palettes(&mut self) -> Vec<Vec<Color>> {
        self.bus.ppu.debug_palettes()
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
        let mut cpu = Cpu::new(build_cartridge(&rom).unwrap(), None, None);
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
