use std::{rc::Rc, cell::RefCell};

use bitflags::bitflags;

bitflags! {
    pub struct Inputs: u8 {
        const A      = (1 << 0);
        const B      = (1 << 1);
        const SELECT = (1 << 2);
        const START  = (1 << 3);
        const UP     = (1 << 4);
        const DOWN   = (1 << 5);
        const LEFT   = (1 << 6);
        const RIGHT  = (1 << 7);
    }
}

#[derive(Debug)]
pub struct Controller {
    inputs: Inputs,
    read_state: u8,
    strobe: bool
}

impl Controller {
    fn new() -> Self {
        Controller {
            inputs: Inputs { bits: 0 },
            read_state: 0,
            strobe: true
        }
    }

    pub fn read(&mut self) -> u8 {
        let bit = self.read_state & 1;
        self.read_state >>= 1;
        bit
    }

    pub fn write(&mut self, value: u8) {
        self.strobe = (value & 1) != 0;
        self.read_state = self.inputs.bits;
    }

    pub fn clear(&mut self) {
        self.inputs.bits = 0;
    }

    pub fn input(&mut self, input: Inputs) {
        self.inputs.insert(input)
    }
    pub fn remove_input(&mut self, input: Inputs) {
        self.inputs.remove(input)
    }
}

pub type ControllerRef = Rc<RefCell<Box<Controller>>>;

pub fn make_controller() -> ControllerRef {
    Rc::new(RefCell::new(Box::new(Controller::new())))
}