mod cart;
mod controller;
mod cpu;
mod error;
mod graphics;
pub mod ines;
mod mem;
mod ppu;

use ines::parse::INesFile;

use crate::cart::builder::build_cartridge;
use crate::controller::make_controller;
use crate::graphics::graphics::GraphicsBuilder;
use cpu::cpu::Cpu;
use std::{error::Error, fs, path::Path};

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Instant;

use clap::Parser;

#[derive(Parser)]
struct CliArgs {
    rom_path: String,
    #[arg(short, long)]
    scale: Option<u32>,
    #[arg(short, long)]
    debug: bool,
}

// #[funtime::timed]
fn main() -> Result<(), Box<dyn Error>> {
    let args = CliArgs::parse();

    let rom = fs::read(Path::new(&args.rom_path))?;
    let ines_rom = INesFile::try_from(&rom).expect("Path provided is not a valid NES ROM.");

    let controller = make_controller();
    let cart = build_cartridge(&ines_rom).expect("This ROM is not supported.");

    if args.debug {
        println!("{:#X?}", ines_rom.header);
        println!("Cartridge type: {}", cart.name());
    }

    let mut cpu = Cpu::new(
        cart, 
        Some(controller.clone()),
        None,
    );

    cpu.reset()?;

    let mut graphics = GraphicsBuilder::new()
        .debug(args.debug)
        .scale(args.scale)
        .build();

    let mut running = true;

    const FPS: f64 = 59.0;

    let mut prev_time = Instant::now();
    let mut residual_time = 0.0;
    let mut paused = false;

    // Main loop
    while running {
        let events = graphics.events();
        // Poll for events
        for event in events {
            match event {
                #[rustfmt::skip]
                Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    running = false;
                }
                #[rustfmt::skip]
                Event::KeyDown {keycode: Some(Keycode::F), ..} | Event::KeyDown { keycode: Some(Keycode::P), ..} => {
                    paused = true;
                    graphics.render_frame(cpu.next_frame()?, &mut cpu)?;
                }
                #[rustfmt::skip]
                Event::KeyDown {keycode: Some(Keycode::Space) ,..} => {
                    paused = !paused;
                    controller.borrow_mut().clear();
                }
                #[rustfmt::skip]
                Event::KeyDown {  keycode: Some(keycode), ..} => {
                    use Keycode::*;
                    let input = match keycode {
                        W => Some(controller::Inputs::UP),
                        A => Some(controller::Inputs::LEFT),
                        S => Some(controller::Inputs::DOWN),
                        D => Some(controller::Inputs::RIGHT),
                        J => Some(controller::Inputs::B),
                        K => Some(controller::Inputs::A),
                        Q => Some(controller::Inputs::SELECT),
                        E => Some(controller::Inputs::START),
                        _ => None
                    };
                    if let Some(input) = input {
                        controller.borrow_mut().input(input);
                    }
                }
                #[rustfmt::skip]
                Event::KeyUp {  keycode: Some(keycode), ..} => {
                    use Keycode::*;
                    let input = match keycode {
                        W => Some(controller::Inputs::UP),
                        A => Some(controller::Inputs::LEFT),
                        S => Some(controller::Inputs::DOWN),
                        D => Some(controller::Inputs::RIGHT),
                        J => Some(controller::Inputs::B),
                        K => Some(controller::Inputs::A),
                        Q => Some(controller::Inputs::SELECT),
                        E => Some(controller::Inputs::START),
                        _ => None
                    };
                    if let Some(input) = input {
                        controller.borrow_mut().remove_input(input);
                    }
                }
                _ => {}
            }
        }
        
        // Generate frame
        if !paused {
            let now = Instant::now();
            let elapsed = (now - prev_time).as_secs_f64();
            if residual_time > 0.0 {
                residual_time -= elapsed;
            } else {
                residual_time += (1.0 / FPS) - elapsed;
                graphics.render_frame(cpu.next_frame()?, &mut cpu)?;
            }
            prev_time = now; 
        }

    }

    Ok(())
}
