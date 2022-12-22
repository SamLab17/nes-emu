use sdl2::EventPump;
use sdl2::pixels::Color;
use sdl2::event::{Event, EventPollIterator};
use sdl2::keyboard::Keycode;
use sdl2::render::Canvas;
use sdl2::video::Window;
use std::time::Duration;

pub struct NesEmuGraphics {
    canvas: Canvas<Window>,
    event_pump: EventPump,
}

impl NesEmuGraphics {
    const WIDTH: u32 = 256;
    const HEIGHT: u32 = 240;
    const DEFAULT_SCALE: u32 = 2;
    const TITLE: &'static str = "nes-emu";

    pub fn new(iscale: u32) -> Self {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();    
        let window = video_subsystem
            .window(Self::TITLE, Self::WIDTH * iscale, Self::HEIGHT * iscale)
            .position_centered()
            .build()
            .unwrap();
        
        let mut canvas = window.into_canvas().build().unwrap();
        canvas.clear();
        let event_pump = sdl_context.event_pump().unwrap();
        
        NesEmuGraphics { canvas, event_pump }
    }

    pub fn events(&mut self) -> EventPollIterator {
        self.event_pump.poll_iter()
    }

    pub fn render_frame(frame: [[Color; Self::WIDTH as usize]; Self::HEIGHT as usize]) {
        todo!("Render frame")
    }

}