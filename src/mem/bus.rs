use std::ops::Range;
use std::vec;

use nom::Map;

use crate::ppu::ppu::Ppu;
use crate::error::Result;
use crate::mem::device::{MemoryDevice, MemoryError};

use crate::cart::{cart::Cartridge, mock::MockCartridge}; 
use super::ram::Ram;

pub struct MemoryBus {
    ram: Ram,
    pub ppu: Ppu,
    pub cart: Cartridge
}

pub struct MemoryBusBuilder {
    ram: Option<Ram>,
    ppu: Ppu,
    cart: Option<Box<dyn MemoryDevice>>
}

impl MemoryBusBuilder {
    pub fn new() -> Self {
        Self {
            ram: None,
            ppu: Ppu::default(),
            cart: None
        }
    }

    pub fn with_ram(mut self, init_ram: Option<&[u8]>) -> Self {
        self.ram = Some(init_ram.map(Ram::from).unwrap_or_else(|| Ram::default()));
        self
    }

    pub fn with_cart(mut self, cart: Box<dyn MemoryDevice>) -> Self {
        self.cart = Some(cart);
        self
    }

    pub fn build(self) -> MemoryBus {
        MemoryBus { 
            ram: self.ram.unwrap_or_else(|| Ram::default()),
            ppu: self.ppu,
            cart: self.cart.unwrap_or_else(|| Box::new(MockCartridge{}))
        }
    }
}

impl MemoryDevice for MemoryBus {

    fn name(&self) -> String { "Memory Bus".into() }

    fn read(&self, addr: u16) -> Result<u8> {
        match addr {
            0x0000..=0x1FFF => self.ram.read(addr),
            0x2000..=0x3FFF => self.ppu.read(addr),
            0x4014 => self.ppu.read(addr),
            0x4020..=0xFFFF => self.cart.read(addr),
            _ => Err(Box::new(MemoryError::InvalidAddress(addr)))
        }
    }

    fn write(&mut self, addr: u16, byte: u8) -> Result<()> {
        match addr {
            0x0000..=0x1FFF => self.ram.write(addr, byte),
            0x2000..=0x3FFF => self.ppu.write(addr, byte),
            0x4014 => self.ppu.write(addr, byte),
            0x4020..=0xFFFF => self.cart.write(addr, byte),
            _ => Err(Box::new(MemoryError::InvalidAddress(addr)))
        }
    }
}
