mod rendering;
mod virtual_machine;

extern crate sdl2;

use clap::Parser;
use std::time::{Duration, Instant};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::keyboard::Scancode::Insert;
use sdl2::pixels::Color;
use crate::rendering::Renderer;
use crate::virtual_machine::VirtualMachine;

#[derive(Parser)]
#[command(version, about = "CHIP-8 Emulator written in rust", long_about = None)]
struct Cli {
	#[arg(help = "the binary file to load into memory")]
	program: std::path::PathBuf,
	#[arg(short, long, help = "target frequency of the emulator, in Hz. emulator will run slightly slower than this.")]
	frequency: Option<u32>
}

fn main() {
	
	let cli = Cli::parse();
	
	let sdl_context = sdl2::init().unwrap();
	let mut renderer = Renderer::build(&sdl_context);
	renderer.canvas.set_draw_color(Color::RGB(91, 206, 250));
	renderer.canvas.clear();
	
	let mut vm = VirtualMachine::build();
	
	let program = std::fs::read(cli.program);
	let program = match program {
		Ok(p) => p,
		Err(e) => panic!("Unable to read binary. Error: {}", e)
	};
	vm.load_program(program);
	
	let do_sleep = cli.frequency.is_some();
	let sleep_time = if do_sleep {
		Duration::new(0, 1_000_000_000 / cli.frequency.unwrap())
	} else { Duration::ZERO	};

	let mut perf_timer = Instant::now();
	let mut perf_counter: u64 = 0;
	let mut cycle_timer = Instant::now();
	let mut event_pump = sdl_context.event_pump().unwrap();
	// here we go!
	'running: loop {
		for event in event_pump.poll_iter() {
			match event {
				Event::Quit { .. } |
				Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'running,
				Event::KeyDown { keycode: Some(keycode), .. } => vm.handle_keydown(keycode),
				Event::KeyUp { keycode: Some(keycode), .. } => vm.handle_keyup(keycode),
				_ => {}
			}
		}
		
		vm.cycle();
		
		if vm.update_display {
			renderer.canvas.set_draw_color(Color::RGB(91, 206, 250));
			renderer.canvas.clear();
			renderer.canvas.set_draw_color(Color::RGB(245, 169, 184));
			renderer.draw_video_memory(vm.video_memory);
			vm.update_display = false;
			renderer.canvas.present();
		}
		
		// run at roughly target frequency
		if do_sleep {
			std::thread::sleep(sleep_time.saturating_sub(cycle_timer.elapsed()));
			cycle_timer = Instant::now();
		}		
		
		// update window title with 500ms average clock rate
		perf_counter += 1;
		if perf_timer.elapsed().as_millis() > 500 {
			let freq = perf_counter as f64 / perf_timer.elapsed().as_secs_f64();
			renderer.canvas.window_mut().set_title(format!("CHIP-8 | {}", format_frequency(freq)).as_str()).unwrap();
			perf_counter = 0;
			perf_timer = Instant::now();
		}
	}
}

fn format_frequency(freq: f64) -> String {
	let (suffix, number) = if freq < 1_000.0 {
		("Hz", freq)
	} else if freq < 1_000_000.0 {
		("KHz", freq / 1_000.0)
	} else if freq < 1_000_000_000.0 {
		("MHz", freq / 1_000_000.0)
	} else {
		("GHz", freq / 1_000_000_000.0)
	};
	
	format!("{number:.2}{suffix}")
}