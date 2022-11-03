use std::fmt::{Debug, Display};
use std::error::Error;

pub type Result<T> = core::result::Result<T, Box<dyn Error>>;