// Copyright (C) 2024 Sasha (WoMspace), All Rights Reserved

use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2::render::WindowCanvas;
use sdl2::Sdl;

pub struct Renderer {
	pub canvas: WindowCanvas,
	foreground: Color,
	background: Color
}

impl Renderer {
	pub fn build(sdl_context: &Sdl) -> Renderer {
		let video_subsystem = sdl_context.video().unwrap();
		let window = video_subsystem
			.window("CHIP-8", 1280, 640)
			.position_centered()
			.build()
			.unwrap();
		let mut canvas = window.into_canvas()
			.accelerated()
			.build()
			.unwrap();
		let _ = canvas.set_logical_size(64, 32);

		Renderer {
			canvas,
			foreground: Color::RGB(255, 255, 255),
			background: Color::RGB(0, 0, 0)
		}
	}

	pub fn draw_video_memory(&mut self, video_buffer: [bool; 2048]) {
		let mut points: Vec<Point> = Vec::with_capacity(2048);
		for (i, pixel) in video_buffer.iter().enumerate() {
			if *pixel {
				let x = (i % 64) as i32;
				let y = (i / 64) as i32;
				let point = Point::new(x, y);
				points.push(point);
			}
		}
		
		self.canvas.set_draw_color(self.background);
		self.canvas.clear();
		self.canvas.set_draw_color(self.foreground);
		let _ = self.canvas.draw_points(points.as_slice());
		self.canvas.present()
	}
	
	pub fn get_colors(&mut self, color: &str) {
		let (fg, bg) = match color {
			"amber" => (Color::RGB(255, 197, 0), Color::RGB(30, 18, 8)),
			"pride" => (Color::RGB(245, 169, 184), Color::RGB(91, 206, 250)),
			"moneybags" => (Color::RGB(239, 152, 21), Color::RGB(196, 196, 196)),
			"mono" => (Color::WHITE, Color::BLACK),
			_ => {
				eprintln!("Unknown color '{}', defaulting to mono", color);
				(Color::WHITE, Color::BLACK)
			},
		};
		self.foreground = fg;
		self.background = bg;
	}
}
