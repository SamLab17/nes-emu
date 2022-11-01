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
        // FIXME: This is wrong!
        Some(self.mem[addr as usize])
    }

    fn write(&mut self, addr: u16, byte: u8) -> Result<(), MemoryError> {
        // FIXME: This is wrong!
        self.mem[addr as usize] = byte;
        Ok(())
    }
}

impl Ram {
    pub fn from(bytes: &[u8]) -> Self {
        let mut r: Self = Default::default();
        r.write_many(0, bytes).expect("bytes too large");
        r
    }
}