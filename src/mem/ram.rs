use super::device::{MemoryDevice, MemoryError};

const RAM_SIZE: usize = 0x1FFF;

pub struct Ram {
    mem: [u8; RAM_SIZE]
}

impl Default for Ram {
    fn default() -> Ram {
        Ram {
            mem: [0; RAM_SIZE]
        }
    }
}

impl MemoryDevice for Ram {
    fn read(&self, addr: u16) -> Option<u8> {
        None
    }

    fn write(&mut self, addr: u16) -> Result<(), MemoryError> {
        Ok(())
    }
}