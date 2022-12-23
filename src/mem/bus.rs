use std::cell::RefCell;
use std::rc::Rc;

use crate::cart::mock::mock_cart;
use crate::ppu::ppu::{Ppu, Frame, PpuBuilder};
use crate::error::Result;
use crate::mem::device::{MemoryDevice, MemoryError};

use crate::cart::{cart::Cartridge, mock::MockCartridge}; 
use super::device::{rd_only, inv_addr};
use super::ram::Ram;

pub struct MemoryBus {
    ram: Ram,
    pub ppu: Ppu,
    pub cart: Rc<RefCell<Cartridge>>
}

pub struct MemoryBusBuilder {
    ram: Option<Ram>,
    cart: Option<Cartridge>
}

impl MemoryBusBuilder {
    pub fn new() -> Self {
        Self {
            ram: None,
            cart: None
        }
    }

    pub fn with_ram(mut self, init_ram: Option<&[u8]>) -> Self {
        self.ram = Some(init_ram.map(Ram::from).unwrap_or_default());
        self
    }

    pub fn with_cart(mut self, cart: Cartridge) -> Self {
        self.cart = Some(cart);
        self
    }

    pub fn build(self) -> MemoryBus {
        let cart =  Rc::new(RefCell::new(self.cart.unwrap_or_else(|| mock_cart())));
        MemoryBus { 
            ram: self.ram.unwrap_or_default(),
            ppu: PpuBuilder::new(cart.clone()).build().unwrap(),
            cart
        }
    }
}

impl MemoryDevice for MemoryBus {

    fn name(&self) -> String { "Memory Bus".into() }

    fn read(&mut self, addr: u16) -> Result<u8> {
        match addr {
            0x0000..=0x1FFF => self.ram.read(addr),
            0x2000..=0x3FFF => self.ppu.read(addr),
            0x4014 => self.ppu.read(addr),
            0x4020..=0xFFFF => self.cart.borrow_mut().read(addr),
            _ => Err(inv_addr(addr))
        }
    }

    fn write(&mut self, addr: u16, byte: u8) -> Result<()> {
        match addr {
            0x0000..=0x1FFF => self.ram.write(addr, byte),
            0x2000..=0x3FFF => self.ppu.write(addr, byte),
            0x4014 => self.ppu.write(addr, byte),
            0x4020..=0xFFFF => self.cart.borrow_mut().write(addr, byte),
            _ => Err(inv_addr(addr))
        }
    }
}

impl MemoryBus {
    pub fn ppu_tick(&mut self) -> Result<Option<Frame>> {
        self.ppu.tick(&mut self.ram)
    }
}
