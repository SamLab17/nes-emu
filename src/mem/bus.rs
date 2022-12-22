use crate::ppu::ppu::Ppu;
use crate::error::Result;
use crate::mem::device::{MemoryDevice, MemoryError};

use crate::cart::{cart::Cartridge, mock::MockCartridge}; 
use super::ram::Ram;

pub struct MemoryBus {
    ram: Ram,
    // TODO: The PPU will also need access to the cartridge (for its CHR RAM),
    // I think the easiest thing to do will be to "clone" the cartridge here
    // and give the PPU its own owned copy. Just need to make sure that there is
    // absolutely no interaction between CPU and PPU with the cartridge (because they'd
    // have separate copies of the cart)

    // OR!! Store the cartridge as a Rc<dyn MemoryDevice> (reference counted)
    // so both can share a reference. However, can they both have a mutable reference?
    // ... doesn't look like it, we would need to use a Mutex to have 2 mutable references

    // Maybe we just let the Ppu borrow the cartridge (as a function argument) 
    // whenever it needs it... :(
    pub ppu: Ppu,
    pub cart: Cartridge
}

pub struct MemoryBusBuilder {
    ram: Option<Ram>,
    ppu: Option<Ppu>,
    cart: Option<Cartridge>
}

impl MemoryBusBuilder {
    pub fn new() -> Self {
        Self {
            ram: None,
            ppu: None,
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

    pub fn with_ppu(mut self, ppu: Ppu) -> Self {
        self.ppu = Some(ppu);
        self
    }

    pub fn build(self) -> MemoryBus {
        MemoryBus { 
            ram: self.ram.unwrap_or_default(),
            ppu: self.ppu.unwrap_or_default(),
            cart: self.cart.unwrap_or_else(|| Box::new(MockCartridge{}))
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
