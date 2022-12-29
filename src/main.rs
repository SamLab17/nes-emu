mod mem;
mod error;
mod cpu;
pub mod ines;
mod ppu;
mod cart;
mod graphics;

use ines::parse::INesFile;

use std::{fs, path::Path, error::Error};
use cpu::cpu::Cpu;
use crate::cart::builder::build_cartridge;
use crate::graphics::graphics::GraphicsBuilder;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Duration;
use std::time::Instant;

use clap::Parser;

#[derive(Parser)]
struct CliArgs {
    rom_path: String,
    #[arg(short, long)]
    scale: Option<u32>,
    #[arg(short, long)]
    debug: bool
}

// #[funtime::timed]
fn main() -> Result<(), Box<dyn Error>> {
    let args = CliArgs::parse();

    let rom = fs::read(Path::new(&args.rom_path))?;
    let ines_rom = INesFile::try_from(&rom).expect("Path provided is not a valid NES ROM.");

    let mut cpu = Cpu::new(
        build_cartridge(&ines_rom).expect("This ROM is not supported."),
    );
    
    cpu.reset()?;

    // let mut graphics = SimpleGraphics::new(3);
    let mut graphics = GraphicsBuilder::new()
    .debug(args.debug)
    .scale(args.scale)
    .build();

    let mut running = true;

    const FPS: f64 = 62.0;

    let start_time = Instant::now();
    let mut paused = false;

    let mut num_frames = 0;
    // Main loop
    while running {
        let pre_time = graphics.performance_counter()?; 

        let events = graphics.events().into_iter().collect::<Vec<Event>>();
        // Poll for events
        for event in events {
            match event {
                Event::Quit{..} | Event::KeyDown { keycode: Some(Keycode::Escape), ..} => {
                    running = false;
                },
                Event::KeyDown { keycode: Some(Keycode::C), ..} => {
                    paused = true;
                    for _ in 0..3 {
                        cpu.system_tick(None)?;
                    }
                    graphics.render_frame(cpu.debug_frame(), &mut cpu)?;
                }
                Event::KeyDown { keycode: Some(Keycode::F), ..} | Event::KeyDown { keycode: Some(Keycode::P), ..} => {
                    paused = true;
                    graphics.render_frame(cpu.next_frame()?, &mut cpu)?;
                },
                Event::KeyDown { keycode: Some(Keycode::Space), ..} => {
                    paused = !paused;
                }
                _ => {}
            }
        }
        // Generate frame
        if !paused {
            graphics.render_frame(cpu.next_frame()?, &mut cpu)?;
            num_frames += 1;
            let curr_time = graphics.performance_counter()?;

            let render_time = (curr_time - pre_time) as f64 / (graphics.performance_frequency()? as f64);
            let sleep = f64::max((1.0/FPS) - render_time, 0.0);
            std::thread::sleep(Duration::from_secs_f64(sleep));
        }
    }

    let elapsed = (Instant::now() - start_time).as_millis();
    println!("Time elapsed: {} ms", elapsed);
    println!("Frames generated: {}", num_frames);
    println!("Avg FPS: {}", (num_frames as f64 * 1000.0) / (elapsed as f64));
    Ok(())
}
