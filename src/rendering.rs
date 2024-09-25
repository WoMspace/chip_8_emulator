// Copyright (C) 2024 Sasha (WoMspace), All Rights Reserved

use std::ffi::{c_char, CStr, CString};
use std::{mem, ptr};
use std::time::Instant;
use gl::{FRAGMENT_SHADER, VERTEX_SHADER};
use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2::render::WindowCanvas;
use sdl2::Sdl;
extern crate gl;
use gl::types::*;
use sdl2::video::GLProfile;

pub struct Renderer {
	pub canvas: WindowCanvas,
	foreground: Color,
	background: Color,
	program: GLProgram,
	start_time: Instant
}

pub struct GLProgram {
	program: GLuint,
	frag_shader: GLuint,
	vert_shader: GLuint,
	vbo: Option<GLuint>,
	vao: Option<GLuint>
}
impl Drop for GLProgram {
	fn drop(&mut self) {
		unsafe {
			gl::DeleteProgram(self.program);
			gl::DeleteShader(self.frag_shader);
			gl::DeleteShader(self.vert_shader);
			if let Some(vao) = self.vbo { gl::DeleteBuffers(1, &vao) }
			if let Some(vbo) = self.vbo { gl::DeleteBuffers(1, &vbo) }
		}
	}
}

impl Renderer {
	pub fn build(sdl_context: &Sdl) -> Renderer {
		let video_subsystem = sdl_context.video().unwrap();
		let window = video_subsystem
			.window("CHIP-8", 1280, 640)
			.position_centered()
			.opengl()
			.build()
			.unwrap();
		let mut canvas = window.into_canvas()
			.accelerated()
			.index(find_sdl_gl_driver().unwrap())
			.build()
			.unwrap();
		let _ = canvas.set_logical_size(64, 32);
		canvas.window().gl_set_context_to_current().unwrap();
		
		let gl_attr = video_subsystem.gl_attr();
		gl_attr.set_context_profile(GLProfile::Core);
		gl_attr.set_context_version(3, 3);
		// load opengl function addresses
		gl::load_with(|name| video_subsystem.gl_get_proc_address(name) as *const _);
		
		assert_eq!(gl_attr.context_profile(), GLProfile::Core);
		assert_eq!(gl_attr.context_version(), (3, 3));
		
		// create vertex shader
		let vert_shader: GLuint = Self::compile_shader(VSH_SOURCE, VERTEX_SHADER);
		// create fragment shader
		let frag_shader: GLuint = Self::compile_shader(FSH_SOURCE, FRAGMENT_SHADER);
		
		// link shaders
		let program = Self::link_program(vert_shader, frag_shader);
		
		let program = GLProgram {
			program,
			frag_shader,
			vert_shader,
			vbo: None,
			vao: None,
		};

		Renderer {
			canvas,
			foreground: Color::RGB(255, 255, 255),
			background: Color::RGB(0, 0, 0),
			program,
			start_time: Instant::now()
		}
	}
	
	fn compile_shader(source: &str, ty: GLenum) -> GLuint {
		let shader;
		unsafe {
			// compile the shader
			shader = gl::CreateShader(ty);
			let c_str = CString::new(source.as_bytes()).unwrap();
			gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
			gl::CompileShader(shader);
			
			// get compile status
			let mut status = gl::FALSE as GLint;
			gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);
			if status != (gl::TRUE as GLint) {
				let mut len = 0;
				gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
				let mut buf: Vec<u8> = Vec::with_capacity(len as usize - 1);
				buf.fill(0);
				buf.set_len((len as usize) - 1);
				gl::GetShaderInfoLog(shader, len, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);
				panic!("{}", std::str::from_utf8(&buf).expect("ShaderInfoLog contains invalid UTF-8"));
			}
		}
		shader
	}
	
	fn link_program(vert_shader: GLuint, frag_shader: GLuint) -> GLuint {
		unsafe {
			// link
			let program = gl::CreateProgram();
			gl::AttachShader(program, vert_shader);
			gl::AttachShader(program, frag_shader);
			gl::LinkProgram(program);
			
			// get link status
			let mut status = gl::FALSE as GLint;
			gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);
			
			// fail on error
			if(status != (gl::TRUE) as GLint) {
				let mut len: GLint = 0;
				gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
				let mut buf = Vec::with_capacity(len as usize);
				buf.fill(0);
				buf.set_len((len as usize) - 1);
				gl::GetProgramInfoLog(program, len, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);
				panic!("{}", std::str::from_utf8(&buf).expect("ProgramInfoLog contains invalid UTF-8"))
			}
			program
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
		// self.canvas.present()
	}
	
	pub fn post_fx(&mut self) {
		self.program.vao = Some(0);
		self.program.vbo = Some(0);
		unsafe { 
			// create vertex array object
			gl::GenVertexArrays(1, &mut self.program.vbo.unwrap());
			gl::BindVertexArray(self.program.vao.unwrap());
			// create vertex buffer object and copy vertex data to it
			gl::GenBuffers(1, &mut self.program.vbo.unwrap());
			gl::BindBuffer(gl::ARRAY_BUFFER, self.program.vbo.unwrap());
			gl::BufferData(
				gl::ARRAY_BUFFER,
				(VERTEX_DATA.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
				mem::transmute(&VERTEX_DATA[0]),
				gl::STATIC_DRAW
			);
			
			// use shader program
			gl::UseProgram(self.program.program);
			gl::BindAttribLocation(self.program.program, 0, CString::new("out_color").unwrap().as_ptr());
			
			// specify layout of the vertex data
			let pos_attr = gl::GetAttribLocation(self.program.program, CString::new("position").unwrap().as_ptr()) as GLuint;
			gl::EnableVertexAttribArray(pos_attr);
			gl::VertexAttribPointer(pos_attr, 2, gl::FLOAT, gl::FALSE, 0, ptr::null());
			
			// bind sampler2D uniform
			let uniform_i = gl::GetUniformLocation(self.program.program, CString::new("i").unwrap().as_ptr());
			gl::Uniform1f(uniform_i, self.start_time.elapsed().as_secs_f32());
			
			// do it!
			gl::DrawArrays(gl::TRIANGLES, 0, 3);
		}
		self.canvas.present();
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

fn find_sdl_gl_driver() -> Option<u32> {
	for (index, item) in sdl2::render::drivers().enumerate() {
		if item.name == "opengl" {
			return Some(index as u32)
		}
	}
	None
}

const FSH_SOURCE: &str = "\
#version 150
uniform sampler2D colortex;
uniform float i;
in vec2 uv;
out vec4 out_color;

void main() {
	// out_color = vec4(uv.x, uv.y, 1.0, 1.0);

	// vec3 color = texture2D(colortex, uv).rgb;
	// out_color = vec4(color, 1.0);

	out_color = vec4(sin(i), cos(i), 1.0, 1.0);
}
";

const VSH_SOURCE: &str = "\
#version 150
in vec2 position;
out vec2 uv;

void main() {
	gl_Position = vec4(position, 0.0, 1.0);
	uv = position;
}
";

const VERTEX_DATA: [GLfloat; 6] = [0.0, 0.5, 0.5, -0.5, -0.5, -0.5];