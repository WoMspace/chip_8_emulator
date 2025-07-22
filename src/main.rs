// Copyright (C) 2024 Sasha (WoMspace), All Rights Reserved

mod rendering;
mod virtual_machine;
mod audio;

extern crate sdl3;

use clap::Parser;
use std::time::{Duration, Instant};
use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use sdl3::pixels::Color;
use crate::audio::AudioPlayer;
use crate::rendering::Renderer;
use crate::virtual_machine::VirtualMachine;

#[derive(Parser)]
#[command(version, about = "CHIP-8 Emulator written in rust", long_about = None)]
struct Cli {
	#[arg(help = "the binary file to load into memory")]
	program: std::path::PathBuf,
	#[arg(short, long, help = "target frequency of the emulator, in Hz. emulator will run slightly slower than this.")]
	frequency: Option<u32>,
	#[arg(short = 'v', long = "verbose", action = clap::ArgAction::Count, help = "print extra debug information, use multiple times for more verbosity")]
	debug: u8,
	#[arg(short, long, help = "colour scheme of the terminal. options are 'mono', 'amber', 'pride', 'moneybags'")]
	colour: Option<String>,
}

fn main() {
	let cli = Cli::parse();
	
	let sdl_context = sdl3::init().unwrap();
	let audio_subsystem = sdl_context.audio().unwrap();
	
	let mut renderer = Renderer::build(&sdl_context);
	renderer.canvas.set_draw_color(Color::RGB(0, 0, 0));
	renderer.canvas.clear();
	if cli.colour.is_some() {
		renderer.get_colors(cli.colour.unwrap().as_str());
	}
	
	let mut audio_player = AudioPlayer::build(audio_subsystem);
	
	let mut vm = VirtualMachine::build();
	vm.debug_level = cli.debug;
	
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
		if vm.sound_timer > 0 {
			audio_player.play()
		} else {
			audio_player.pause()
		}
		
		if vm.update_display {
			renderer.draw_video_memory(vm.video_memory);
			vm.update_display = false;
		}
		
		// run at roughly target frequency
		if do_sleep {
			let mut resume = false;
			while !resume {
				resume = cycle_timer.elapsed() >= sleep_time;
			}
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
		("kHz", freq / 1_000.0)
	} else if freq < 1_000_000_000.0 {
		("MHz", freq / 1_000_000.0)
	} else {
		("GHz", freq / 1_000_000_000.0)
	};
	
	format!("{number:.2} {suffix}")
}