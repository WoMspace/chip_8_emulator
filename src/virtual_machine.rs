use rand::rngs::ThreadRng;
use rand::{thread_rng, Rng};
use sdl2::keyboard::Keycode;

pub struct VirtualMachine {
	memory: [u8; 4096],
	video_memory: [bool; 2048],
	program_counter: u16,
	index_register: u16,
	stack: Vec<u16>,
	delay_timer: u8,
	sound_timer: u8,
	registers: [u8; 8],
	pub keys: [bool; 16],
	rng: ThreadRng,
	pub update_display: bool,
}

struct Opcodes {
	pub i: u16,
	pub x: u16,
	pub y: u16,
	pub n: u16,
	pub nn: u8,
	pub nnn: u16,
}

#[allow(non_snake_case)]
impl VirtualMachine {
	pub fn build() -> VirtualMachine {
		let mut vm = VirtualMachine {
			memory: [0; 4096],
			video_memory: [false; 2048],
			program_counter: 0x200,
			index_register: 0,
			stack: Vec::new(),
			delay_timer: 0,
			sound_timer: 0,
			registers: [0; 8],
			keys: [false; 16],
			rng: thread_rng(),
			update_display: false,
		};

		// copy font into memory
		for (i, byte) in VirtualMachine::FONT.iter().enumerate() {
			vm.memory[0x50 + i] = *byte;
		}

		vm
	}

	fn fetch(&mut self) -> Opcodes {
		let a = self.memory[self.program_counter as usize];
		let b = self.memory[self.program_counter as usize + 1];
		self.program_counter += 2;
		let instruction = ((a as u16) << 8) | (b as u16);
		let i = (instruction & 0xF000) >> 12;
		let x = (instruction & 0x0F00) >> 8;
		let y = (instruction & 0x00F0) >> 4;
		let n = instruction & 0x000F;
		let nn = (instruction & 0x00FF) as u8;
		let nnn = instruction & 0x0FFF;

		Opcodes {
			i,
			x,
			y,
			n,
			nn,
			nnn,
		}
	}

	pub fn handle_keydown(&mut self, keycode: Keycode) {
		match keycode {
			Keycode::Num1 => self.keys[0x1] = true,
			Keycode::Num2 => self.keys[0x2] = true,
			Keycode::Num3 => self.keys[0x3] = true,
			Keycode::Num4 => self.keys[0xC] = true,
			Keycode::Q => self.keys[0x4] = true,
			Keycode::W => self.keys[0x5] = true,
			Keycode::E => self.keys[0x6] = true,
			Keycode::R => self.keys[0xD] = true,
			Keycode::A => self.keys[0x7] = true,
			Keycode::S => self.keys[0x8] = true,
			Keycode::D => self.keys[0x9] = true,
			Keycode::F => self.keys[0xE] = true,
			Keycode::Z => self.keys[0xA] = true,
			Keycode::X => self.keys[0x0] = true,
			Keycode::C => self.keys[0xB] = true,
			Keycode::V => self.keys[0xF] = true,
			_ => {}
		}
	}

	pub fn handle_keyup(&mut self, keycode: Keycode) {
		match keycode {
			Keycode::Num1 => self.keys[0x1] = false,
			Keycode::Num2 => self.keys[0x2] = false,
			Keycode::Num3 => self.keys[0x3] = false,
			Keycode::Num4 => self.keys[0xC] = false,
			Keycode::Q => self.keys[0x4] = false,
			Keycode::W => self.keys[0x5] = false,
			Keycode::E => self.keys[0x6] = false,
			Keycode::R => self.keys[0xD] = false,
			Keycode::A => self.keys[0x7] = false,
			Keycode::S => self.keys[0x8] = false,
			Keycode::D => self.keys[0x9] = false,
			Keycode::F => self.keys[0xE] = false,
			Keycode::Z => self.keys[0xA] = false,
			Keycode::X => self.keys[0x0] = false,
			Keycode::C => self.keys[0xB] = false,
			Keycode::V => self.keys[0xF] = false,
			_ => {}
		}
	}

	fn op_00E0(&mut self) {
		// CLS: clear display
		self.video_memory.fill(false);
	}

	fn op_00EE(&mut self) {
		// RET: return from subroutine
		self.program_counter = self.stack.pop().unwrap();
	}

	fn op_1nnn(&mut self, opcodes: Opcodes) {
		// JP addr: jump to location nnn
		self.program_counter = opcodes.nnn;
	}

	fn op_2nnn(&mut self, opcodes: Opcodes) {
		// CALL addr: call subroutine at nnn
		self.stack.push(self.program_counter);
		self.program_counter = opcodes.nnn;
	}

	fn op_3xkk(&mut self, opcodes: Opcodes) {
		// SE Vx, byte: skip next instruction if register Vx == kk
		if self.registers[opcodes.x as usize] == opcodes.nn {
			self.program_counter += 2
		}
	}

	fn op_4xkk(&mut self, opcodes: Opcodes) {
		// SNE Vx, byte: skip next instruction if register Vx != kk
		if self.registers[opcodes.x as usize] != opcodes.nn {
			self.program_counter += 2
		}
	}

	fn op_5xy0(&mut self, opcodes: Opcodes) {
		// SE Vx, Vy: skip next instruction if registers Vx == Vy
		if self.registers[opcodes.x as usize] == self.registers[opcodes.y as usize] {
			self.program_counter += 2
		}
	}

	fn op_6xkk(&mut self, opcodes: Opcodes) {
		// LD Vx, byte: set register Vx to byte kk
		self.registers[opcodes.x as usize] = opcodes.nn;
	}

	fn op_7xkk(&mut self, opcodes: Opcodes) {
		// ADD Vx, byte: add value kk to register Vx, storing in Vx
		self.registers[opcodes.x as usize] += opcodes.nn;
	}

	fn op_8xy0(&mut self, opcodes: Opcodes) {
		// LD Vx, Vy: set register Vx to the value in register Vy
		self.registers[opcodes.x as usize] = self.registers[opcodes.y as usize];
	}

	fn op_8xy1(&mut self, opcodes: Opcodes) {
		// OR Vx, Vy: bitwise OR registers Vx and Vy, storing result in Vx
		self.registers[opcodes.x as usize] |= self.registers[opcodes.y as usize];
	}

	fn op_8xy2(&mut self, opcodes: Opcodes) {
		// AND Vx, Vy: bitwise AND registers Vx and Vy, storing result in Vx
		self.registers[opcodes.x as usize] &= self.registers[opcodes.y as usize];
	}

	fn op_8xy3(&mut self, opcodes: Opcodes) {
		// XOR Vx, Vy: bitwise XOR registers Vx and Vy, storing result in Vx
		self.registers[opcodes.x as usize] ^= self.registers[opcodes.y as usize];
	}

	fn op_8xy4(&mut self, opcodes: Opcodes) {
		// ADD Vx, Vy: add registers Vx and Vy, storing result in Vx. if overflow, set VF to 1, otherwise 0
		let result: u32 =
			self.registers[opcodes.x as usize] as u32 + self.registers[opcodes.y as usize] as u32;
		if result > 255 {
			self.registers[opcodes.x as usize] = (result % 255) as u8;
			self.registers[0xF] = 1;
		} else {
			self.registers[opcodes.x as usize] = result as u8;
			self.registers[0xF] = 0;
		}
	}

	fn op_8xy5(&mut self, opcodes: Opcodes) {
		// SUB Vx, Vy: subtract register Vy from Vx, storing result in Vx. if Vx > Vy, set VF to 1, otherwise 0
		let set_flag = self.registers[opcodes.x as usize] > self.registers[opcodes.y as usize];
		self.registers[0xF] = if set_flag { 1 } else { 0 };
		self.registers[opcodes.x as usize] -= self.registers[opcodes.y as usize];
	}

	fn op_8xy6(&mut self, opcodes: Opcodes) {
		// SHR Vx {, Vy}: store the value in register Vx in Vy, then right shift register Vx by one, storing the lost bit in VF
		self.registers[opcodes.y as usize] = self.registers[opcodes.x as usize];
		self.registers[opcodes.x as usize] >>= 1;
		self.registers[0xF] = self.registers[opcodes.y as usize] & 0x0001;
	}

	fn op_8xy7(&mut self, opcodes: Opcodes) {
		// SUBN Vx, Vy: subtract register Vx from Vy, storing result in Vx. if Vy > Vx, set VF to 1, otherwise 0
		let set_flag = self.registers[opcodes.y as usize] > self.registers[opcodes.x as usize];
		self.registers[0xF] = if set_flag { 1 } else { 0 };
		self.registers[opcodes.x as usize] =
			self.registers[opcodes.y as usize] - self.registers[opcodes.x as usize];
	}

	fn op_8xyE(&mut self, opcodes: Opcodes) {
		// SHL Vx {, Vy} // store the value in register Vx in Vy, then left shift register Vx by one, storing the lost bit in VF
		self.registers[opcodes.y as usize] = self.registers[opcodes.x as usize];
		self.registers[opcodes.x as usize] <<= 1;
		self.registers[0xF] = (self.registers[opcodes.y as usize] & 0x0080) >> 7;
	}

	fn op_9xy0(&mut self, opcodes: Opcodes) {
		// SNE Vx, Vy: Skip next instruction if register Vx != Vy
		if self.registers[opcodes.x as usize] != self.registers[opcodes.y as usize] {
			self.program_counter += 2;
		}
	}

	fn op_Annn(&mut self, opcodes: Opcodes) {
		// LD I, addr: set index register to nnn
		self.index_register = opcodes.nnn;
	}

	fn op_Bnnn(&mut self, opcodes: Opcodes) {
		// JP V0, addr: Jump to the address nnn + the value in register V0
		self.program_counter = opcodes.nnn + self.registers[0] as u16;
	}

	fn op_Cxkk(&mut self, opcodes: Opcodes) {
		// RND Vx, byte: generate a random number, AND with value nn, store in register Vx
		self.registers[opcodes.x as usize] = self.rng.gen::<u8>() & opcodes.nn;
	}

	fn op_Dxyn(&mut self, opcodes: Opcodes) {
		// DRW Vx, Vy, nibble: display an n-byte sprite - starting at index register - at location Vx, Vy. if any pixels are XORed off, flag register VF is set to 1, otherwise 0
		// sprite starting position should wrap, but sprites themselves should clip
		self.update_display = true;
		let x = self.registers[opcodes.x as usize] % 64;
		let y = self.registers[opcodes.y as usize] % 32;
		self.registers[0xF] = 0;
		let mut bitmask = 128u8; // bitmask: 1000 0000
		let range = (self.index_register as usize)..(opcodes.n as usize);
		for (col_i, sprite_row) in self.memory[range].iter().enumerate() {
			for row_i in 0..8 {
				// extract the bit from memory
				let bit = (sprite_row & bitmask) > 0;
				if y as usize + col_i > 31 || x as usize + row_i > 63 {
					continue;
				} // discard draws outside the screen
				let video_index = (y as usize + col_i) * 64 + x as usize + row_i;
				if self.video_memory[video_index] & bit == true {
					// set VF if pixels will be XORed off
					self.registers[0xF] = 1;
				}
				// do the XOR
				self.video_memory[video_index] ^= bit;
				bitmask >>= 1;
			}
		}
	}

	fn op_Ex9E(&mut self, opcodes: Opcodes) {
		// SKP Vx: skip the next instruction if the key with value Vx is pressed
		let key = self.registers[opcodes.x as usize] as usize;
		if self.keys[key] { self.program_counter += 2 }
	}
	
	fn op_ExA1(&mut self, opcodes: Opcodes) {
		// SKNP Vx: skip the next instruction if the key with value Vx is NOT pressed
		let key = self.registers[opcodes.x as usize] as usize;
		if !self.keys[key] { self.program_counter += 2 }
	}
	
	fn op_Fx07(&mut self, opcodes: Opcodes) {
		// LD Vx, DT: set register Vx to value in delay timer
		self.registers[opcodes.x as usize] = self.delay_timer;
	}
	
	fn op_Fx0A(&mut self, opcodes: Opcodes) {
		// LD Vx, K: block until any keypress, store key in register Vx
		'blocking: loop {
			for (id, key) in self.keys.iter().enumerate() {
				if *key {
					self.registers[opcodes.x as usize] = id as u8;
					break 'blocking;
				}
			}
		}
		todo!("switch to repeating the instruction, rather than completely blocking. timers should decrement, etc.")
	}
	
	fn op_Fx15(&mut self, opcodes: Opcodes) {
		// LD DT, Vx: set delay timer to value in register Vx
		self.delay_timer = self.registers[opcodes.x as usize];
	}
	
	fn op_Fx18(&mut self, opcodes: Opcodes) {
		// LD ST, Vx: set sound timer to value in register Vx
		self.sound_timer = self.registers[opcodes.x as usize];
	}
	
	fn op_Fx1E(&mut self, opcodes: Opcodes) {
		// ADD I, Vx: set register I to I + Vx
		self.index_register += self.registers[opcodes.x as usize] as u16;
		// this is sussy behaviour. original cosmac interpreter did not do this, but the amiga chip-8 interpreter did, and one known game relies on this behaviour
		self.registers[0xF] = if self.index_register > 0x0FFF { 1 } else { 0 }
	}
	
	fn op_Fx29(&mut self, opcodes: Opcodes) {
		// LD F, Vx: set index register to location of font for digit Vx
		todo!("ligma balls lmao gottem")
	}

	const FONT: [u8; 80] = [
		0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
		0x20, 0x60, 0x20, 0x20, 0x70, // 1
		0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
		0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
		0x90, 0x90, 0xF0, 0x10, 0x10, // 4
		0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
		0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
		0xF0, 0x10, 0x20, 0x40, 0x40, // 7
		0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
		0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
		0xF0, 0x90, 0xF0, 0x90, 0x90, // A
		0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
		0xF0, 0x80, 0x80, 0x80, 0xF0, // C
		0xE0, 0x90, 0x90, 0x90, 0xE0, // D
		0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
		0xF0, 0x80, 0xF0, 0x80, 0x80, // F
	];
}
