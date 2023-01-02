use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use crate::cart::mock::mock_cart;
use crate::controller::ControllerRef;
use crate::error::Result;
use crate::ppu::ppu::{Ppu, PpuBuilder};

use super::error::inv_addr;
use super::ram::Ram;
use crate::cart::cart::Cartridge;

pub struct MemoryBus {
    ram: Ram,
    pub ppu: Ppu,
    pub cart: Arc<Mutex<Cartridge>>,
    p1: Option<ControllerRef>,
    p2: Option<ControllerRef>
}

pub struct MemoryBusBuilder {
    ram: Option<Ram>,
    cart: Option<Cartridge>,
    p1: Option<ControllerRef>,
    p2: Option<ControllerRef>
}

impl MemoryBusBuilder {
    pub fn new() -> Self {
        Self {
            ram: None,
            cart: None,
            p1: None,
            p2: None
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

    pub fn with_controllers(mut self, p1: Option<ControllerRef>, p2: Option<ControllerRef>) -> Self {
        self.p1 = p1;
        self.p2 = p2;
        self
    }

    pub fn build(self) -> MemoryBus {
        let cart = Arc::new(Mutex::new(self.cart.unwrap_or_else(|| mock_cart())));
        MemoryBus {
            ram: self.ram.unwrap_or_default(),
            ppu: PpuBuilder::new(cart.clone()).build().unwrap(),
            cart,
            p1: self.p1,
            p2: self.p2
        }
    }
}

impl MemoryBus {
    pub fn read(&mut self, addr: u16) -> Result<u8> {
        match addr {
            0x0000..=0x1FFF => self.ram.read(addr),
            0x2000..=0x3FFF => self.ppu.read(addr),
            0x4000..=0x4015 => Ok(0), 
            0x4016 => {
                if let Some(p1) = self.p1.as_ref() {
                    Ok(p1.lock().unwrap().read())
                    // Ok(p1.borrow_mut().read())
                } else {
                    Ok(0)
                }
            }
            0x4017 => {
                if let Some(p2) = self.p2.as_ref() {
                    Ok(p2.lock().unwrap().read())
                } else {
                    Ok(0)
                }
            }
            0x4020..=0xFFFF => {
                self.cart.lock().unwrap().read(addr)
            },
            _ => Err(inv_addr(addr)),
        }
    }

    pub fn write(&mut self, addr: u16, byte: u8) -> Result<()> {
        match addr {
            0x0000..=0x1FFF => self.ram.write(addr, byte),
            0x2000..=0x3FFF => self.ppu.write(addr, byte),
            0x4016 => {
                if let Some(p1) = self.p1.as_ref() {
                    p1.lock().unwrap().write(byte);
                }
                Ok(())
            },
            0x4017 => {
                if let Some(p2) = self.p2.as_ref() {
                    p2.lock().unwrap().write(byte);
                }
                Ok(())
            }
            0x4000..=0x4015 => Ok(()),
            0x4020..=0xFFFF => self.cart.lock().unwrap().write(addr, byte),
            _ => Err(inv_addr(addr)),
        }
    }
}
