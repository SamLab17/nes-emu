mod mem;
mod error;
mod cpu;
pub mod ines;
mod ppu;
mod cart;
mod graphics;

use ines::parse::INesFile;

use std::{env, fs, path::Path, error::Error};
use cpu::cpu::Cpu;
use graphics::NesEmuGraphics;

use crate::cart::builder::build_cartridge;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Duration;

fn main() -> Result<(), Box<dyn Error>> {
    let rom_path = env::args().nth(1).expect("No ROM file provided");
    let rom = fs::read(Path::new(&rom_path))?;

    let ines_rom = INesFile::try_from(&rom).expect("Path provided is not a valid NES ROM.");

    let mut cpu = Cpu::new(
        build_cartridge(&ines_rom).expect("This ROM is not supported."),
    );

    let mut graphics = NesEmuGraphics::new(3);

    let mut running = true;

    // Main loop
    while running {

        // Poll for events
        for event in graphics.events() {
            match event {
                Event::Quit{..} | Event::KeyDown { keycode: Some(Keycode::Escape), ..} => {
                    running = false;
                },
                _ => {}
            }
        }
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    Ok(())
}
