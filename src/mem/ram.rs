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

impl MemoryDevice for Ram {

    fn name(&self) -> String { "RAM".into() }

    fn read(&self, addr: u16) -> Result<u8> {
        if addr > 0x1FFF {
            Err(Box::new(MemoryError::InvalidAddress(addr)))
        } else {
            Ok(self.mem[(addr & (RAM_SIZE-1)) as usize])
        }
    }

    fn write(&mut self, addr: u16, byte: u8) -> Result<()> {
        if addr > 0x1FFF {
            Err(Box::new(MemoryError::InvalidAddress(addr)))
        } else {
            self.mem[(addr & (RAM_SIZE-1)) as usize] = byte;
            Ok(())
        }
    }
}

impl Ram {
    pub fn from(bytes: &[u8]) -> Self {
        if bytes.len() > RAM_SIZE as usize{
            panic!("RAM size is smaller than bytes specified.")
        }
        let mut r: Self = Default::default();
        r.write_many(0, bytes).expect("bytes too large");
        r
    }
}
