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

fn main() {
	let mut renderer = Renderer::build();
	renderer.canvas.set_draw_color(Color::RGB(91, 206, 250));
	renderer.canvas.clear();
	
	let mut rng = rand::thread_rng();
	
	let mut event_pump = renderer.sdl_context.event_pump().unwrap();
	'running: loop {
		renderer.canvas.set_draw_color(Color::RGB(91, 206, 250));
		renderer.canvas.clear();
		for event in event_pump.poll_iter() {
			match event {
				Event::Quit {..} |
				Event::KeyDown { keycode: Some(Keycode::Escape), ..} => {
					break 'running
				},
				_ => {}
			}
		}
		renderer.canvas.set_draw_color(Color::RGB(245, 169, 184));
		
		renderer.canvas.present();
		std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
	}
}