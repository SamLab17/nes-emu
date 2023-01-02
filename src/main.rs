mod cart;
mod controller;
mod cpu;
mod error;
mod graphics;
pub mod ines;
mod mem;
mod ppu;

use graphics::graphics::{NesGraphics, CpuInfo};
use ines::parse::INesFile;
use ppu::ppu::Frame;
use sdl2::audio::{AudioSpecDesired, AudioCallback, AudioSpec};

use crate::cart::builder::build_cartridge;
use crate::controller::make_controller;
use crate::graphics::graphics::GraphicsBuilder;
use cpu::cpu::Cpu;
use std::sync::mpsc::{channel, Sender, TryRecvError};
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

fn map_inputs(keycode: &Keycode) -> Option<controller::Inputs> {
    use Keycode::*;
    match keycode {
        W => Some(controller::Inputs::UP),
        A => Some(controller::Inputs::LEFT),
        S => Some(controller::Inputs::DOWN),
        D => Some(controller::Inputs::RIGHT),
        J => Some(controller::Inputs::B),
        K => Some(controller::Inputs::A),
        Q => Some(controller::Inputs::SELECT),
        E => Some(controller::Inputs::START),
        _ => None
    }
}

struct EmuMain {
    cpu: Cpu,
    frame_send: Sender<(Frame, CpuInfo)>,
    audio_spec: AudioSpec,
    // time_step: f64,
    // global_time: f64,
}

impl AudioCallback for EmuMain {
    type Channel = f32;

    fn callback(&mut self, channels: &mut [Self::Channel]) {
        loop {
            let (frame, audio_sample) = self.cpu.system_tick(None).unwrap();
            if let Some(frame) = frame {
                self.frame_send.send((frame, self.cpu.get_info())).unwrap();
            }
            if let Some(audio_sample) = audio_sample {
                channels[0] = audio_sample as f32;
                break;
            }
        }
        // self.global_time += self.time_step;
    }
}
 

fn main() -> Result<(), Box<dyn Error>> {
    let args = CliArgs::parse();

    let rom = fs::read(Path::new(&args.rom_path))?;
    let ines_rom = INesFile::try_from(&rom).expect("Path provided is not a valid NES ROM.");

    if args.debug {
        println!("{:#X?}", ines_rom.header);
    }

    let controller = make_controller();
    let cart = build_cartridge(&ines_rom).expect("This ROM is not supported.");

    if args.debug {
        println!("Cartridge type: {}", cart.name());
    }

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut graphics = GraphicsBuilder::new(video_subsystem)
        .debug(args.debug)
        .scale(args.scale)
        .build();
    
    let audio = sdl_context.audio().unwrap();
    let desired = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(1),
        samples: Some(128)
    };

    let c = controller.clone();
    let (send, rcv) = channel();

    let device = audio.open_playback(None, &desired, move |spec| {
        let mut cpu = Cpu::new(
            cart,
            spec.freq as f64 / spec.samples as f64,
            Some(c),
            None,
        );
        cpu.reset().unwrap();
        println!("{spec:?}");
        EmuMain { cpu, frame_send: send, audio_spec: spec } 
    }).unwrap();
    
    let mut running = true;

    let mut paused = false;
    device.resume();

    // Main loop
    while running {
        let events = event_pump.poll_iter().collect::<Vec<Event>>();
        // Poll for events
        for event in events.iter() {
            match event {
                #[rustfmt::skip]
                Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    running = false;
                }
                #[rustfmt::skip]
                #[rustfmt::skip]
                Event::KeyDown {keycode: Some(Keycode::Space) ,..} => {
                    paused = !paused;
                    if paused {
                        device.pause();
                    } else {
                        device.resume();
                    }
                    controller.lock().unwrap().clear();
                }
                #[rustfmt::skip]
                Event::KeyDown {  keycode: Some(keycode), ..} => {
                    if let Some(input) = map_inputs(keycode) {
                        controller.lock().unwrap().input(input);
                    }
                }
                #[rustfmt::skip]
                Event::KeyUp {  keycode: Some(keycode), ..} => {
                    if let Some(input) = map_inputs(keycode) {
                        controller.lock().unwrap().remove_input(input);
                    }
                }
                _ => {}
            }
        }
        graphics.process_events(&events);
        // println!("waiting on frame...");
        match rcv.try_recv() {
            Ok((frame, info)) => graphics.render_frame(frame, info),
            Err(TryRecvError::Empty) => Ok(()),
            Err(e) => Err(e.into())
        }?;
    }

    device.close_and_get_callback();
    Ok(())
}
