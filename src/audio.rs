// Copyright (C) 2024 Sasha (WoMspace), All Rights Reserved

use sdl3::audio::{AudioCallback, AudioStream, AudioSpec, AudioFormat, AudioStreamWithCallback};
use sdl3::{AudioSubsystem};

struct SquareWave {
	phase_inc: f32,
	phase: f32,
	volume: f32
}

impl AudioCallback<f32> for SquareWave {
	fn callback(&mut self, stream: &mut AudioStream, requested: i32) {
		let mut samples = Vec::with_capacity(requested as usize);
		for _ in 0..requested {
			let sample = if self.phase <= 0.5 {
				self.volume
			} else {
				-self.volume
			};
			samples.push(sample);
			self.phase = (self.phase + self.phase_inc) % 1.0;
		}
		stream.put_data_f32(&samples).unwrap()
	}
}

pub struct AudioPlayer {
	stream: AudioStreamWithCallback<SquareWave>
}

impl AudioPlayer {
	pub fn build(audio_subsystem: AudioSubsystem) -> AudioPlayer {
		let desired_spec = AudioSpec {
			freq: Some(48000),
			channels: Some(1),
			format: Some(AudioFormat::f32_sys())
		};
		let stream = audio_subsystem.open_playback_stream(&desired_spec, 
    SquareWave {
				phase_inc: 440.0 / desired_spec.freq.unwrap() as f32,
				phase : 0.0,
				volume: 0.1
			}).unwrap();
		
		AudioPlayer { stream }
	}
	
	pub fn play(&mut self) {
		let _ = self.stream.resume();
	}
	
	pub fn pause(&mut self) {
		let _ = self.stream.pause();
	}
}