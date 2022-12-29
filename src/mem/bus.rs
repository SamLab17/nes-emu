use std::cell::RefCell;
use std::rc::Rc;

use crate::cart::mock::mock_cart;
use crate::error::Result;
use crate::ppu::ppu::{Ppu, PpuBuilder};

use super::error::inv_addr;
use super::ram::Ram;
use crate::cart::cart::Cartridge;

pub struct MemoryBus {
    ram: Ram,
    pub ppu: Ppu,
    pub cart: Rc<RefCell<Cartridge>>,
}

pub struct MemoryBusBuilder {
    ram: Option<Ram>,
    cart: Option<Cartridge>,
}

impl MemoryBusBuilder {
    pub fn new() -> Self {
        Self {
            ram: None,
            cart: None,
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
        let cart = Rc::new(RefCell::new(self.cart.unwrap_or_else(|| mock_cart())));
        MemoryBus {
            ram: self.ram.unwrap_or_default(),
            ppu: PpuBuilder::new(cart.clone()).build().unwrap(),
            cart,
        }
    }
}

impl MemoryBus {
    pub fn read(&mut self, addr: u16) -> Result<u8> {
        match addr {
            0x0000..=0x1FFF => self.ram.read(addr),
            0x2000..=0x3FFF => self.ppu.read(addr),
            0x4014 => self.ppu.read(addr),
            0x4000..=0x4017 => Ok(0), /*todo!("Read APU")*/
            0x4020..=0xFFFF => self.cart.borrow_mut().read(addr),
            _ => Err(inv_addr(addr)),
        }
    }

    pub fn write(&mut self, addr: u16, byte: u8) -> Result<()> {
        match addr {
            0x0000..=0x1FFF => self.ram.write(addr, byte),
            0x2000..=0x3FFF => self.ppu.write(addr, byte, &self.ram),
            0x4014 => self.ppu.write(addr, byte, &self.ram),
            0x4000..=0x4017 => Ok(()),
            0x4020..=0xFFFF => self.cart.borrow_mut().write(addr, byte),
            _ => Err(inv_addr(addr)),
        }
    }
}
