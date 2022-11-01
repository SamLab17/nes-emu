use crate::mem::device::{MemoryDevice, MemoryError};
use crate::mem::ram::Ram;

struct Mapping {
    start_addr : u16,
    end_addr: u16,
    dev: Box<dyn MemoryDevice>
}

pub struct MemoryBus {
    devices: Vec<Mapping>
}

impl MemoryBus {

    pub fn new() -> Self {
        let r: Ram = Default::default();
        MemoryBus { 
            devices: vec!(
                Mapping {
                    start_addr: 0x0000,
                    end_addr: 0x1FFF,
                    dev: Box::new(r)
                },
            ) 
        }
    }
}

impl MemoryDevice for MemoryBus {
    fn read(&self, addr: u16) -> Option<u8> {
        for dev in self.devices.iter() {
            if addr >= dev.start_addr && addr < dev.end_addr {
                return dev.dev.read(addr)
            }
        }
        None
    }

    fn write(&mut self, addr: u16, byte: u8) -> Result<(), MemoryError> {
        for dev in self.devices.iter_mut() {
            if addr >= dev.start_addr && addr < dev.end_addr {
                return dev.dev.write(addr, byte)
            }
        }
        Err(MemoryError::InvalidAddress(addr))
    }
}