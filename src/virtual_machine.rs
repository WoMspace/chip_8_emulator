use rand::{random, thread_rng, Rng};

struct VirtualMachine {
	memory: [u8; 4096],
	video_memory: [bool; 2048],
	program_counter: u16,
	index_register: u16,
	stack: Vec<u16>,
	delay_timer: u8,
	sound_timer: u8,
	registers: [u8; 8]
}

struct Opcodes {
	pub i: u16,
	pub x: u16,
	pub y: u16,
	pub n: u16,
	pub nn: u8,
	pub nnn: u16
}

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
			registers: [0; 8]
		};
		
		// copy font into memory
		for (i, byte) in VirtualMachine::FONT.iter().enumerate() {
			vm.memory[0x50 + i] = *byte;
		}
		
		vm
	}
	
	pub fn fetch(&mut self) -> Opcodes {
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
		
		Opcodes {i, x, y, n, nn, nnn}
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
		if self.registers[opcodes.x as usize] == opcodes.nn { self.program_counter += 2 }
	}
	
	fn op_4xkk(&mut self, opcodes: Opcodes) { 
		// SNE Vx, byte: skip next instruction if register Vx != kk
		if self.registers[opcodes.x as usize] != opcodes.nn { self.program_counter += 2 }
	}
	
	fn op_5xy0(&mut self, opcodes: Opcodes) { 
		// SE Vx, Vy: skip next instruction if registers Vx == Vy
		if self.registers[opcodes.x as usize] == self.registers[opcodes.y as usize] { self.program_counter += 2 }
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
		let result: u32 = self.registers[opcodes.x as usize] as u32 + self.registers[opcodes.y as usize] as u32;
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
		self.registers[opcodes.x as usize] = self.registers[opcodes.y as usize] - self.registers[opcodes.x as usize];
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
		self.registers[opcodes.x as usize] = random::<u8>() & opcodes.nn;
	}
	
	fn op_Dxyn(&mut self, opcodes: Opcodes) {
		// DRW Vx, Vy, nibble: display an n-byte sprite - starting at index register - at location Vx, Vy. if any pixels are XORed off, flag register VF is set to 1, otherwise 0
		// sprite starting position should wrap, but sprites themselves should clip
		let x = self.registers[opcodes.x as usize] % 64;
		let y = self.registers[opcodes.y as usize] % 32;
		self.registers[0xF] = 0;
		let range = (self.index_register as usize)..(opcodes.n as usize);
		for row in self.memory[range].iter() {
			todo!("bit stuff in DRAW SPRITE instructtion")
		}
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
		0xF0, 0x80, 0xF0, 0x80, 0x80  // F
	];
}