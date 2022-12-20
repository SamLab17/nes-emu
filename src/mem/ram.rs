use super::device::{MemoryDevice, MemoryError};
use crate::error::Result;

const RAM_SIZE: usize = 0x1FFF;

pub struct Ram {
    mem: [u8; RAM_SIZE],
}

impl Default for Ram {
    fn default() -> Ram {
        Ram { mem: [0; RAM_SIZE] }
    }
}

impl MemoryDevice for Ram {

    fn name(&self) -> String { "RAM".into() }

    fn read(&self, addr: u16) -> Result<u8> {
        // FIXME: This is wrong!
        Ok(self.mem[addr as usize])
    }

    fn write(&mut self, addr: u16, byte: u8) -> Result<()> {
        // FIXME: This is wrong!
        self.mem[addr as usize] = byte;
        Ok(())
    }
}

impl Ram {
    pub fn from(bytes: &[u8]) -> Self {
        if bytes.len() > RAM_SIZE {
            panic!("RAM size is smaller than bytes specified.")
        }
        let mut r: Self = Default::default();
        r.write_many(0, bytes).expect("bytes too large");
        r
    }
}
