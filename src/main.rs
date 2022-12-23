mod mem;
mod error;
mod cpu;
pub mod ines;
mod ppu;
mod cart;
mod graphics;

use ines::parse::INesFile;

use core::num;
use std::cmp::min;
use std::ops::Sub;
use std::process::exit;
use std::{env, fs, path::Path, error::Error};
use cpu::cpu::Cpu;
use graphics::NesEmuGraphics;

use crate::cart::builder::build_cartridge;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Duration;
use std::time::Instant;

// #[funtime::timed]
fn main() -> Result<(), Box<dyn Error>> {
    let rom_path = env::args().nth(1).expect("No ROM file provided");
    let rom = fs::read(Path::new(&rom_path))?;

    let ines_rom = INesFile::try_from(&rom).expect("Path provided is not a valid NES ROM.");

    let mut cpu = Cpu::new(
        build_cartridge(&ines_rom).expect("This ROM is not supported."),
    );

    let mut graphics = NesEmuGraphics::new(3);

    let mut running = true;

    const FPS: u32 = 60;
    const NANOS_PER_FRAME: u32 = 1_000_000_000 / FPS;

    let mut prev_time = Instant::now();
    let start_time = Instant::now();
    let mut paused = false;

    let mut num_frames = 0;
    // Main loop
    while running {

        let events = graphics.events().into_iter().collect::<Vec<Event>>();
        // Poll for events
        for event in events {
            match event {
                Event::Quit{..} | Event::KeyDown { keycode: Some(Keycode::Escape), ..} => {
                    running = false;
                },
                Event::KeyDown { keycode: Some(Keycode::P), ..} => {
                    // graphics.render_frame(cpu.next_frame()?)?;
                    cpu.system_tick()?;
                    graphics.render_frame(cpu.debug_frame())?;
                }
                Event::KeyDown { keycode: Some(Keycode::C), ..} => {
                    for _ in 0..300 {
                        cpu.system_tick()?;
                    }
                    graphics.render_frame(cpu.debug_frame())?;
                }
                Event::KeyDown { keycode: Some(Keycode::F), ..} => {
                    graphics.render_frame(cpu.next_frame()?)?;
                },
                Event::KeyDown { keycode: Some(Keycode::Space), ..} => {
                    paused = !paused;
                    prev_time = Instant::now();
                }
                _ => {}
            }
        }
        // Generate frame
        if !paused {
            graphics.render_frame(cpu.next_frame()?)?;
            num_frames += 1;

            let curr_time = Instant::now();
            let time_to_render = curr_time - prev_time;
            let sleep = (NANOS_PER_FRAME as u128) - min(time_to_render.as_nanos(), NANOS_PER_FRAME as u128);
            prev_time = curr_time;
            ::std::thread::sleep(Duration::new(0, sleep.try_into().unwrap()));
        }
    }

    let elapsed = (Instant::now() - start_time).as_millis();
    println!("Time elapsed: {} ms", elapsed);
    println!("Frames generated: {}", num_frames);
    println!("Avg FPS: {}", (num_frames as f64 * 1000.0) / (elapsed as f64));
    Ok(())
}
