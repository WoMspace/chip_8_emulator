mod rendering;
mod virtual_machine;

extern crate sdl2;

use std::time::Duration;
use rand::Rng;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Point;
use crate::rendering::Renderer;
use crate::virtual_machine::VirtualMachine;

fn main() {
	let sdl_context = sdl2::init().unwrap();
	let mut renderer = Renderer::build(&sdl_context);
	renderer.canvas.set_draw_color(Color::RGB(91, 206, 250));
	renderer.canvas.clear();
	
	let mut vm = VirtualMachine::build();

	let mut event_pump = sdl_context.event_pump().unwrap();
	'running: loop {
		renderer.canvas.set_draw_color(Color::RGB(91, 206, 250));
		renderer.canvas.clear();
		for event in event_pump.poll_iter() {
			match event {
				Event::Quit { .. } |
				Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'running,
				Event::KeyDown { keycode: Some(keycode), .. } => vm.handle_keydown(keycode),
				Event::KeyUp { keycode: Some(keycode), .. } => vm.handle_keyup(keycode),
				_ => {}
			}
		}
		renderer.canvas.set_draw_color(Color::RGB(245, 169, 184));

		renderer.canvas.present();
		std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
	}
}