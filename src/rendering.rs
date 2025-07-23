// Copyright (C) 2024 Sasha (WoMspace), All Rights Reserved

use sdl3::gpu::{ColorTargetDescription, ColorTargetInfo, Device, FillMode, GraphicsPipeline, GraphicsPipelineTargetInfo, LoadOp, PrimitiveType, ShaderFormat, ShaderStage, StoreOp};
use sdl3::pixels::Color;
use sdl3::video::Window;
use sdl3::Sdl;

pub struct Renderer {
	gpu: Device,
	pipeline: GraphicsPipeline,
	pub(crate) window: Window,
	foreground: Color,
	background: Color,
}

impl Renderer {
	pub fn build(sdl_context: &Sdl) -> Renderer {
		let video_subsystem = sdl_context.video().unwrap();
		let window = video_subsystem
			.window("CHIP-8", 1280, 640)
			.build()
			.unwrap();
		
		let gpu = Device::new(ShaderFormat::SpirV, cfg!(debug_assertions)).unwrap()
			.with_window(&window).unwrap();
		
		let vert_shader = gpu
			.create_shader()
			.with_code(ShaderFormat::SpirV, include_bytes!("shaders/shader.vert.spv"), ShaderStage::Vertex)
			.with_entrypoint(c"main")
			.build().unwrap();
		
		let frag_shader = gpu
			.create_shader()
			.with_code(ShaderFormat::SpirV, include_bytes!("shaders/shader.frag.spv"), ShaderStage::Fragment)
			.with_entrypoint(c"main")
			.with_uniform_buffers(2)
			.build().unwrap();
		
		let swapchain_format = gpu.get_swapchain_texture_format(&window);
		let pipeline = gpu
			.create_graphics_pipeline()
			.with_vertex_shader(&vert_shader)
			.with_fragment_shader(&frag_shader)
			.with_primitive_type(PrimitiveType::TriangleStrip)
			.with_fill_mode(FillMode::Fill)
			.with_target_info(
				GraphicsPipelineTargetInfo::new().with_color_target_descriptions(&[ColorTargetDescription::new().with_format(swapchain_format)])
			)
			.build().unwrap();

		Renderer {
			gpu,
			pipeline,
			window,
			foreground: Color::RGB(255, 255, 255),
			background: Color::RGB(0, 0, 0)
		}
	}
	
	/// GPU-backed render
	pub fn gpu_draw(&mut self, video_buffer: &[bool; 2048]) {
		let mut command_buffer = self.gpu.acquire_command_buffer().unwrap();
		if let Ok(swapchain) = command_buffer.wait_and_acquire_swapchain_texture(&self.window) {
			let color_targets = [
				ColorTargetInfo::default()
					.with_texture(&swapchain)
					.with_load_op(LoadOp::Clear)
					.with_store_op(StoreOp::Store)
					.with_clear_color(self.background)
			];
			let render_pass = self.gpu.begin_render_pass(&command_buffer, &color_targets, None).unwrap();
			// screen cleared here due to ColorTargetInfo
			render_pass.bind_graphics_pipeline(&self.pipeline);
			let vram = Self::pack_vram(&video_buffer);
			command_buffer.push_fragment_uniform_data(0, &vram);
			let colors = Self::pack_colors(self.foreground, self.background);
			command_buffer.push_fragment_uniform_data(1, &colors);
			render_pass.draw_primitives(5, 1, 0, 0);
			self.gpu.end_render_pass(render_pass);
			command_buffer.submit().unwrap();
		} else {
			// swapchain unavailable, cancel work (?)
			command_buffer.cancel();
			eprintln!("Swapchain unavailable, cancelled command buffer")
		}
	}
	
	// pack the bool array into ints
	fn pack_vram(video_buffer: &[bool; 2048]) -> [u32; 64] {
		let mut out = [0u32; 64];
		
		for x in 0..64 {
			let mut col = 0;
			for y in 0..32 {
				let b = video_buffer[y*64 + x] as u32;
				col |= b << y;
			}
			out[x] = col;
		}
		
		out
	}
	
	// pack the colors into vec4-padded vec3s
	fn pack_colors(fg: Color, bg: Color) -> [f32; 8] {
		[
			fg.r as f32 / 256.0,
			fg.g as f32 / 256.0,
			fg.b as f32 / 256.0,
			0.0,
			bg.r as f32 / 256.0,
			bg.g as f32 / 256.0,
			bg.b as f32 / 256.0,
			0.0,
		]
	}
	
	pub fn get_colors(&mut self, color: &str) {
		let (fg, bg) = match color {
			"amber" => (Color::RGB(255, 197, 0), Color::RGB(30, 18, 8)),
			"pride" => (Color::RGB(245, 169, 184), Color::RGB(91, 206, 250)),
			"moneybags" => (Color::RGB(239, 152, 21), Color::RGB(196, 196, 196)),
			"mono" => (Color::WHITE, Color::BLACK),
			"inverted" => (Color::BLACK, Color::WHITE),
			_ => {
				eprintln!("Unknown colorscheme '{}', defaulting to mono", color);
				(Color::WHITE, Color::BLACK)
			},
		};
		self.foreground = fg;
		self.background = bg;
	}
}
