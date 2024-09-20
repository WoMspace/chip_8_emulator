use sdl2::audio::{AudioCallback, AudioDevice, AudioSpecDesired};
use sdl2::Sdl;

struct SquareWave {
	phase_inc: f32,
	phase: f32,
	volume: f32
}

impl AudioCallback for SquareWave {
	type Channel = f32;

	fn callback(&mut self, out: &mut [f32]) {
		// generate a square wave
		for x in out.iter_mut() {
			*x = if self.phase <= 0.5 {
				self.volume
			} else {
				-self.volume
			};
			self.phase = (self.phase + self.phase_inc) % 1.0;
		}
	}
}

pub struct AudioPlayer {
	audio_device: AudioDevice<SquareWave>
}

impl AudioPlayer {
	pub fn build(sdl_context: &Sdl) -> AudioPlayer {
		let audio_subsystem = sdl_context.audio().unwrap();
		let desired_spec = AudioSpecDesired {
			freq: Some(48000),
			channels: Some(1),
			samples: None
		};
		let device = audio_subsystem.open_playback(None, &desired_spec, |spec| {
			SquareWave {
				phase_inc: 440.0 / spec.freq as f32,
				phase : 0.0,
				volume: 0.1
			}
		}).unwrap();
		AudioPlayer {
			audio_device: device
		}
	}
	
	pub fn play(&mut self) {
		self.audio_device.resume();
	}
	
	pub fn pause(&mut self) {
		self.audio_device.pause();
	}
}