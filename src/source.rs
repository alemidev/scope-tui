use libpulse_binding::{sample::{Spec, Format}, def::BufferAttr, error::PAErr, stream::Direction};
use libpulse_simple_binding::Simple;

pub trait DataSource {
	fn recv(&mut self) -> Option<&[u8]>; // TODO convert in Result and make generic error
}

pub struct PulseAudioSimple {
	simple: Simple,
	buffer: Vec<u8>,
}

impl PulseAudioSimple {
	pub fn new(
		device: Option<&str>, channels: u8, rate: u32, buffer: u32, server_buffer: u32
	) -> Result<Self, PAErr> {
		let spec = Spec {
			format: Format::S16NE, // TODO allow more formats?
			channels, rate,
		};
		if !spec.is_valid() {
			return Err(PAErr(0)); // TODO what error number should we throw?
		}
		let attrs = BufferAttr {
			maxlength: server_buffer * buffer,
			fragsize: buffer,
			..Default::default()
		};
		let simple = Simple::new(
			None,                // Use the default server
			"scope-tui",         // Our application’s name
			Direction::Record,   // We want a record stream
			device,              // Use requested device, or default
			"data",              // Description of our stream
			&spec,               // Our sample format
			None,                // Use default channel map
			Some(&attrs),        // Our hints on how to handle client/server buffers
		)?;
		Ok(Self { simple, buffer: vec![0; buffer as usize] })
	}
}

impl DataSource for PulseAudioSimple {
	fn recv(&mut self) -> Option<&[u8]> {
		match self.simple.read(&mut self.buffer) {
			Ok(()) => Some(&self.buffer),
			Err(e) => {
				eprintln!("[!] could not receive from pulseaudio: {}", e);
				None
			}
		}
	}
}
