use std::fmt::{Debug, Display};

use crate::cpu::reg::Registers;


pub trait NesEmuError: Debug + Display {

}