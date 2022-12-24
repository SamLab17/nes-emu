use super::device::{MemoryDevice, MemoryError};
use crate::error::Result;

const RAM_SIZE: u16 = 0x800;

pub struct Ram {
    mem: [u8; RAM_SIZE as usize],
}

impl Default for Ram {
    fn default() -> Ram {
        Ram { mem: [0; RAM_SIZE as usize] }
    }
}

impl Ram {

}

impl Ram {
    pub fn read(&self, addr: u16) -> Result<u8> {
        if addr > 0x1FFF {
            Err(Box::new(MemoryError::InvalidAddress(addr)))
        } else {
            Ok(self.mem[(addr & (RAM_SIZE-1)) as usize])
        }
    }

    pub fn write(&mut self, addr: u16, byte: u8) -> Result<()> {
        if addr > 0x1FFF {
            Err(Box::new(MemoryError::InvalidAddress(addr)))
        } else {
            self.mem[(addr & (RAM_SIZE-1)) as usize] = byte;
            Ok(())
        }
    }

    pub fn from(bytes: &[u8]) -> Self {
        if bytes.len() > RAM_SIZE as usize{
            panic!("RAM size is smaller than bytes specified.")
        }
        let mut r: Self = Default::default();
        let mut addr = 0;
        for byte in bytes.iter() {
            r.mem[addr] = *byte;
            addr += 1
        }
        r
    }
}
