use std::fmt::{Debug, Display};
use std::error::Error;

use crate::cpu::isa::Instr;

pub type Result<T> = core::result::Result<T, Box<dyn Error>>;