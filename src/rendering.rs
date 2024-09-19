use sdl2::rect::Point;
use sdl2::render::WindowCanvas;
use sdl2::Sdl;

pub struct Renderer {
	pub canvas: WindowCanvas,
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

		let _ = self.canvas.draw_points(points.as_slice());
	}
}
