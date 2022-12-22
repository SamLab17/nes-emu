use std::{collections::HashMap, fs, path::Path};

use crate::error::Result;
use sdl2::pixels::Color;

pub type ColorMap = HashMap<u8, Color>;

const NUM_COLORS: u8 = 0x40;
const DEFAULT_PALETTE_FILE: &'static str = "palette/ntsc.pal";

static DEFAULT_PALETTE:  &'static [u8] = include_bytes!("../../palette/ntsc.pal");

pub fn load_default_color_map() -> ColorMap {
    load_color_map(None).unwrap()
}

pub fn load_color_map(pal_file: Option<&str>) -> Result<ColorMap> {
    let bytes = match pal_file {
        Some(f) => fs::read(Path::new(f))?,
        None => DEFAULT_PALETTE.to_vec()
    };

    if bytes.len() != (NUM_COLORS as usize) * 3 {
        Err("Invalid .pal file".into())
    } else {
        let mut m = HashMap::new();
        let mut iter = bytes.into_iter();
        for color in 0..NUM_COLORS {
            m.insert(
                color,
                Color::RGB(
                    iter.next().unwrap(),
                    iter.next().unwrap(),
                    iter.next().unwrap(),
                ),
            );
        }
        Ok(m)
    }
}
