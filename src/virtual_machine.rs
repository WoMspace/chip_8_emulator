use std::time::{Duration, Instant};
use rand::rngs::ThreadRng;
use rand::{thread_rng, Rng};
use sdl2::keyboard::Keycode;

pub struct VirtualMachine {
	memory: [u8; 4096],
	pub video_memory: [bool; 2048],
	program_counter: u16,
	index_register: u16,
	stack: Vec<u16>,
	delay_timer: u8,
	pub sound_timer: u8,
	timer_counter: Instant,
	registers: [u8; 16],
	pub keys: [bool; 16],
	rng: ThreadRng,
	pub update_display: bool,
	pub debug_level: u8,
}

struct Opcode {
	pub instruction: u16,
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
			timer_counter: Instant::now(),
			registers: [0; 16],
			keys: [false; 16],
			rng: thread_rng(),
			update_display: false,
			debug_level: 0,
		};
		// copy font into memory
		for (i, byte) in VirtualMachine::FONT.iter().enumerate() {
			vm.memory[0x50 + i] = *byte;
		}
		vm
	}
	
	pub fn load_program(&mut self, program: Vec<u8>) {
		if program.len() > 3584 { panic!("Program is too big. Max 3584 bytes, recieved {} bytes", program.len())}
		let range = 0x200..(0x200+program.len());
		self.memory[range].copy_from_slice(&program);
		self.program_counter = 0x200;
	}

	fn fetch_decode(&mut self) -> Opcode {
		let a = self.memory[self.program_counter as usize];
		let b = self.memory[self.program_counter as usize + 1];
		let instruction = ((a as u16) << 8) | (b as u16);
		let i = (instruction & 0xF000) >> 12;
		let x = (instruction & 0x0F00) >> 8;
		let y = (instruction & 0x00F0) >> 4;
		let n = instruction & 0x000F;
		let nn = (instruction & 0x00FF) as u8;
		let nnn = instruction & 0x0FFF;

		Opcode {
			instruction,
			i,
			x,
			y,
			n,
			nn,
			nnn,
		}
	}
	
	pub fn cycle(&mut self) {
		let opcode = self.fetch_decode();
		// println!("PC:{:04X} I:{:01X} Il:{:04X}", self.program_counter, opcode.i, opcode.instruction);
		self.print_debug(&opcode);
		self.decrement_timers();
		self.program_counter += 2;
		self.execute(opcode);
	}
	
	fn print_debug(&mut self, opcode: &Opcode) {
		if self.debug_level == 0 { return }
		let mut output = String::new();
		if self.debug_level > 0 {
			// print PC and current instruction
			output += format!("PC:0x{:04X} I:0x{:04X}", self.program_counter, opcode.instruction).as_str()
		}
		if self.debug_level > 1 {
			// also print index register and regular register
			output += format!(" rI:{:04X}\nr0:0x{:04X} r1:0x{:04X} r2:0x{:04X} r3:0x{:04X} r4:0x{:04X} r5:0x{:04X} r6:0x{:04X} r7:0x{:04X}\nr8:0x{:04X} r9:0x{:04X} rA:0x{:04X} rB:0x{:04X} rC:0x{:04X} rD:0x{:04X} rE:0x{:04X} rF:0x{:04X}",
			                  self.index_register, self.registers[0x0], self.registers[0x1], self.registers[0x2], self.registers[0x3], self.registers[0x4],
			                  self.registers[0x5], self.registers[0x6], self.registers[0x7], self.registers[0x8], self.registers[0x9], self.registers[0xA],
			                  self.registers[0xB], self.registers[0xC], self.registers[0xD], self.registers[0xE], self.registers[0xF]).as_str()
		}
		
		println!("{output}")
	}
	
	fn decrement_timers(&mut self) {
		if self.timer_counter.elapsed() > Duration::from_nanos(1_000_000_000 / 60) {
			self.sound_timer = self.sound_timer.saturating_sub(1);
			self.delay_timer = self.delay_timer.saturating_sub(1);
			self.timer_counter = Instant::now();
		}
	}
	
	fn execute(&mut self, opcode: Opcode) {
		match opcode.i {
			// 0x0 => { if opcode.n == 0 { self.op_00E0() } else { self.op_00EE() } }
			0x0 => match opcode.nn {
				0xE0 => self.op_00E0(),
				0xEE => self.op_00EE(),
				_ => panic!("Unknown instruction {:04X}", opcode.instruction)
			}
			0x1 => self.op_1nnn(opcode),
			0x2 => self.op_2nnn(opcode),
			0x3 => self.op_3xkk(opcode),
			0x4 => self.op_4xkk(opcode),
			0x5 => self.op_5xy0(opcode),
			0x6 => self.op_6xkk(opcode),
			0x7 => self.op_7xkk(opcode),
			0x8 => match opcode.n {
				0x0 => self.op_8xy0(opcode),
				0x1 => self.op_8xy1(opcode),
				0x2 => self.op_8xy2(opcode),
				0x3 => self.op_8xy3(opcode),
				0x4 => self.op_8xy4(opcode),
				0x5 => self.op_8xy5(opcode),
				0x6 => self.op_8xy6(opcode),
				0x7 => self.op_8xy7(opcode),
				0xE => self.op_8xyE(opcode),
				_ => panic!("Unknown instruction {:04X}", opcode.instruction)
			}
			0x9 => self.op_9xy0(opcode),
			0xA => self.op_Annn(opcode),
			0xB => self.op_Bnnn(opcode),
			0xC => self.op_Cxkk(opcode),
			0xD => self.op_Dxyn(opcode),
			0xE => match opcode.nn {
				0x9E => self.op_Ex9E(opcode),
				0xA1 => self.op_ExA1(opcode),
				_ => panic!("Unknown instruction {:04X}", opcode.instruction)
			},
			0xF => match opcode.nn {
				0x07 => self.op_Fx07(opcode),
				0x0A => self.op_Fx0A(opcode),
				0x15 => self.op_Fx15(opcode),
				0x18 => self.op_Fx18(opcode),
				0x1E => self.op_Fx1E(opcode),
				0x29 => self.op_Fx29(opcode),
				0x33 => self.op_Fx33(opcode),
				0x55 => self.op_Fx55(opcode),
				0x65 => self.op_Fx65(opcode),
				_ => panic!("Unknown instruction {:04X}", opcode.instruction)
			}
			_ => panic!("Unknown instruction {:04X}", opcode.instruction)
		}
	}

	pub fn handle_keydown(&mut self, keycode: Keycode) {
		if let Keycode::Num1 = keycode { self.keys[0x1] = true	}
		if let Keycode::Num2 = keycode { self.keys[0x2] = true	}
		if let Keycode::Num3 = keycode { self.keys[0x3] = true	}
		if let Keycode::Num4 = keycode { self.keys[0xC] = true	}
		if let Keycode::Q = keycode { self.keys[0x4] = true	}
		if let Keycode::W = keycode { self.keys[0x5] = true	}
		if let Keycode::E = keycode { self.keys[0x6] = true	}
		if let Keycode::R = keycode { self.keys[0xD] = true	}
		if let Keycode::A = keycode { self.keys[0x7] = true	}
		if let Keycode::S = keycode { self.keys[0x8] = true	}
		if let Keycode::D = keycode { self.keys[0x9] = true	}
		if let Keycode::F = keycode { self.keys[0xE] = true	}
		if let Keycode::Z = keycode { self.keys[0xA] = true	}
		if let Keycode::X = keycode { self.keys[0x0] = true	}
		if let Keycode::C = keycode { self.keys[0xB] = true	}
		if let Keycode::V = keycode { self.keys[0xF] = true }
	}

	pub fn handle_keyup(&mut self, keycode: Keycode) {
		if let Keycode::Num1 = keycode { self.keys[0x1] = false }
		if let Keycode::Num2 = keycode { self.keys[0x2] = false }
		if let Keycode::Num3 = keycode { self.keys[0x3] = false }
		if let Keycode::Num4 = keycode { self.keys[0xC] = false }
		if let Keycode::Q = keycode { self.keys[0x4] = false }
		if let Keycode::W = keycode { self.keys[0x5] = false }
		if let Keycode::E = keycode { self.keys[0x6] = false }
		if let Keycode::R = keycode { self.keys[0xD] = false }
		if let Keycode::A = keycode { self.keys[0x7] = false }
		if let Keycode::S = keycode { self.keys[0x8] = false }
		if let Keycode::D = keycode { self.keys[0x9] = false }
		if let Keycode::F = keycode { self.keys[0xE] = false }
		if let Keycode::Z = keycode { self.keys[0xA] = false }
		if let Keycode::X = keycode { self.keys[0x0] = false }
		if let Keycode::C = keycode { self.keys[0xB] = false }
		if let Keycode::V = keycode { self.keys[0xF] = false }
	}

	fn op_00E0(&mut self) {
		// CLS: clear display
		self.video_memory.fill(false);
		self.update_display = true;
	}

	fn op_00EE(&mut self) {
		// RET: return from subroutine
		self.program_counter = self.stack.pop().unwrap();
	}

	fn op_1nnn(&mut self, opcode: Opcode) {
		// JP addr: jump to location nnn
		self.program_counter = opcode.nnn;
	}

	fn op_2nnn(&mut self, opcode: Opcode) {
		// CALL addr: call subroutine at nnn
		self.stack.push(self.program_counter);
		self.program_counter = opcode.nnn;
	}

	fn op_3xkk(&mut self, opcode: Opcode) {
		// SE Vx, byte: skip next instruction if register Vx == kk
		if self.registers[opcode.x as usize] == opcode.nn {
			self.program_counter += 2
		}
	}

	fn op_4xkk(&mut self, opcode: Opcode) {
		// SNE Vx, byte: skip next instruction if register Vx != kk
		if self.registers[opcode.x as usize] != opcode.nn {
			self.program_counter += 2
		}
	}

	fn op_5xy0(&mut self, opcode: Opcode) {
		// SE Vx, Vy: skip next instruction if registers Vx == Vy
		if self.registers[opcode.x as usize] == self.registers[opcode.y as usize] {
			self.program_counter += 2
		}
	}

	fn op_6xkk(&mut self, opcode: Opcode) {
		// LD Vx, byte: set register Vx to byte kk
		self.registers[opcode.x as usize] = opcode.nn;
	}

	fn op_7xkk(&mut self, opcode: Opcode) {
		// ADD Vx, byte: add value kk to register Vx, storing in Vx
		let (result, _) = self.registers[opcode.x as usize].overflowing_add(opcode.nn);
		self.registers[opcode.x as usize] = result;
	}

	fn op_8xy0(&mut self, opcode: Opcode) {
		// LD Vx, Vy: set register Vx to the value in register Vy
		self.registers[opcode.x as usize] = self.registers[opcode.y as usize];
	}

	fn op_8xy1(&mut self, opcode: Opcode) {
		// OR Vx, Vy: bitwise OR registers Vx and Vy, storing result in Vx
		self.registers[opcode.x as usize] |= self.registers[opcode.y as usize];
		self.registers[0xF] = 0;
	}

	fn op_8xy2(&mut self, opcode: Opcode) {
		// AND Vx, Vy: bitwise AND registers Vx and Vy, storing result in Vx
		self.registers[opcode.x as usize] &= self.registers[opcode.y as usize];
		self.registers[0xF] = 0;
	}

	fn op_8xy3(&mut self, opcode: Opcode) {
		// XOR Vx, Vy: bitwise XOR registers Vx and Vy, storing result in Vx
		self.registers[opcode.x as usize] ^= self.registers[opcode.y as usize];
		self.registers[0xF] = 0;
	}

	fn op_8xy4(&mut self, opcode: Opcode) {
		// ADD Vx, Vy: add registers Vx and Vy, storing result in Vx. if overflow, set VF to 1, otherwise 0
		let result = self.registers[opcode.x as usize].overflowing_add(self.registers[opcode.y as usize]);
		self.registers[opcode.x as usize] = result.0;
		self.registers[0xF] = if result.1 { 1 } else { 0 };
	}

	fn op_8xy5(&mut self, opcode: Opcode) {
		// SUB Vx, Vy: subtract register Vy from Vx, storing result in Vx. if Vx > Vy, set VF to 1, otherwise 0
		let (result, not_flag) = self.registers[opcode.x as usize].overflowing_sub(self.registers[opcode.y as usize]);
		self.registers[opcode.x as usize] = result;
		self.registers[0xF] = if not_flag { 0 } else { 1 };
	}

	fn op_8xy6(&mut self, opcode: Opcode) {
		// SHR Vx ,Vy: store the value in register Vy in Vx, then right shift register Vx by one, storing the lost bit in VF
		self.registers[0xF] = self.registers[opcode.y as usize] & 0x1;
		self.registers[opcode.x as usize] = self.registers[opcode.y as usize] >> 1;
	}

	fn op_8xy7(&mut self, opcode: Opcode) {
		// SUBN Vx, Vy: subtract register Vx from Vy, storing result in Vx. if Vy > Vx, set VF to 1, otherwise 0		
		let (result, not_flag) = self.registers[opcode.y as usize].overflowing_sub(self.registers[opcode.x as usize]);
		self.registers[opcode.x as usize] = result;
		self.registers[0xF] = if not_flag { 0 } else { 1 }
	}

	fn op_8xyE(&mut self, opcode: Opcode) {
		// SHL Vx, Vy // store the value in register Vy in Vx, then left shift register Vx by one, storing the lost bit in VF
		self.registers[0xF] = (self.registers[opcode.y as usize] & 0x80) >> 7;
		self.registers[opcode.x as usize] = self.registers[opcode.y as usize] << 1;
	}

	fn op_9xy0(&mut self, opcode: Opcode) {
		// SNE Vx, Vy: Skip next instruction if register Vx != Vy
		if self.registers[opcode.x as usize] != self.registers[opcode.y as usize] {
			self.program_counter += 2;
		}
	}

	fn op_Annn(&mut self, opcode: Opcode) {
		// LD I, addr: set index register to nnn
		self.index_register = opcode.nnn;
	}

	fn op_Bnnn(&mut self, opcode: Opcode) {
		// JP V0, addr: Jump to the address nnn + the value in register V0
		self.program_counter = opcode.nnn + self.registers[0] as u16;
	}

	fn op_Cxkk(&mut self, opcode: Opcode) {
		// RND Vx, byte: generate a random number, AND with value nn, store in register Vx
		self.registers[opcode.x as usize] = self.rng.gen::<u8>() & opcode.nn;
	}

	fn op_Dxyn(&mut self, opcode: Opcode) {
		// DRW Vx, Vy, nibble: display an n-byte sprite - starting at index register - at location Vx, Vy. if any pixels are XORed off, flag register VF is set to 1, otherwise 0
		// sprite starting position should wrap, but sprites themselves should clip
		self.update_display = true;
		let x = self.registers[opcode.x as usize] % 64;
		let y = self.registers[opcode.y as usize] % 32;
		self.registers[0xF] = 0;
		let bitmask = 0x80; // bitmask: 1000 0000
		let range = (self.index_register as usize)..(self.index_register as usize + opcode.n as usize);
		for (col_i, sprite_row) in self.memory[range].iter().enumerate() {
			for row_i in 0..8 {
				// extract the bit from memory
				let bit = (sprite_row & (bitmask >> row_i)) != 0;
				if y as usize + col_i > 31 || x as usize + row_i > 63 {
					continue;
				} // discard draws outside the screen
				let video_index = ((y as usize + col_i) * 64) + (x as usize + row_i);
				if self.video_memory[video_index] & bit {
					// set VF if pixels will be XORed off
					self.registers[0xF] = 1;
				}
				// do the XOR
				self.video_memory[video_index] ^= bit;
				// self.video_memory[video_index] = bit;
				// bitmask >>= 1;
			}
		}
	}

	fn op_Ex9E(&mut self, opcode: Opcode) {
		// SKP Vx: skip the next instruction if the key with value Vx is pressed
		let key = self.registers[opcode.x as usize] as usize;
		if self.keys[key] { self.program_counter += 2 }
	}
	
	fn op_ExA1(&mut self, opcode: Opcode) {
		// SKNP Vx: skip the next instruction if the key with value Vx is NOT pressed
		let key = self.registers[opcode.x as usize] as usize;
		if !self.keys[key] { self.program_counter += 2 }
	}
	
	fn op_Fx07(&mut self, opcode: Opcode) {
		// LD Vx, DT: set register Vx to value in delay timer
		self.registers[opcode.x as usize] = self.delay_timer;
	}
	
	fn op_Fx0A(&mut self, opcode: Opcode) {
		// LD Vx, K: block until any keypress, store key in register Vx
		if self.keys.iter().any(|k| *k){
			for (id, key) in self.keys.iter().enumerate() {
				if *key {
					self.registers[opcode.x as usize] = id as u8;
				}
			}
		} else { self.program_counter -= 2 }
	}
	
	fn op_Fx15(&mut self, opcode: Opcode) {
		// LD DT, Vx: set delay timer to value in register Vx
		self.delay_timer = self.registers[opcode.x as usize];
	}
	
	fn op_Fx18(&mut self, opcode: Opcode) {
		// LD ST, Vx: set sound timer to value in register Vx
		self.sound_timer = self.registers[opcode.x as usize];
	}
	
	fn op_Fx1E(&mut self, opcode: Opcode) {
		// ADD I, Vx: set register I to I + Vx
		self.index_register += self.registers[opcode.x as usize] as u16;
		// this is sussy behaviour. original cosmac interpreter did not do this, but the amiga chip-8 interpreter did, and one known game relies on this behaviour
		self.registers[0xF] = if self.index_register > 0x0FFF { 1 } else { 0 }
	}
	
	fn op_Fx29(&mut self, opcode: Opcode) {
		// LD F, Vx: set index register to location of font for digit Vx
		// font starts at 0x50, each char is 5 bytes long
		let digit = self.registers[opcode.x as usize] as u16;
		self.index_register = 0x50 + (digit * 5);
	}
	
	fn op_Fx33(&mut self, opcode: Opcode) {
		// LD B, Vx: separate digits from value in register Vx and store them in memory at locations I, I+1, and I+2
		let mut value = self.registers[opcode.x as usize];
		let ones = value % 10;
		self.memory[self.index_register as usize + 2] = ones;
		value /= 10;
		let tens = value % 10;
		self.memory[self.index_register as usize + 1] = tens;
		value /= 10;
		let hundreds = value % 10;
		self.memory[self.index_register as usize] = hundreds;
	}
	
	fn op_Fx55(&mut self, opcode: Opcode) {
		// LD [I], Vx: store registers V0 through Vx in memory starting at index register
		let range = 0..=opcode.x as usize;
		for (i, reg) in self.registers[range].iter().enumerate() {
			self.memory[self.index_register as usize + i] = *reg;
		}
		self.index_register += opcode.x + 1;
	}
	
	fn op_Fx65(&mut self, opcode: Opcode) {
		// LD Vx, [I]: read memory starting at index register into registers V0 through Vx
		let range = 0..=opcode.x as usize;
		for i in range {
			self.registers[i] = self.memory[self.index_register as usize + i];
		}
		self.index_register += opcode.x + 1;
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
