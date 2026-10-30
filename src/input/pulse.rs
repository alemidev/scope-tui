use libpulse_binding::{
	def::BufferAttr, error::{Code, PAErr}, sample::{Format, Spec}, stream::Direction,
};
use libpulse_simple_binding::Simple;

use super::{
	format::{SampleParser, Signed16PCM},
	stream_to_matrix,
};

pub struct PulseSource {
	simple: Simple,
	buffer: Vec<u8>,
	channels: usize,
}

impl super::Source for PulseSource {
	fn recv(&mut self) -> Option<super::SampleTape> {
		match self.simple.read(&mut self.buffer) {
			Ok(()) => Some(stream_to_matrix(
				self.buffer.chunks(2).map(Signed16PCM::parse),
				self.channels,
				32768.0,
			)),
			Err(e) => {
				eprintln!("[!] could not receive from pulseaudio: {}", e);
				None
			}
		}
	}
}

#[derive(Default)]
pub struct PulseHost;

impl super::Host for PulseHost {
	type Err = PAErr;
	type Src = PulseSource;
	type Cfg = crate::cfg::PulseOptions;

	fn header() -> Vec<String> {
		vec![]
	}

	fn list(&self) -> Result<Vec<Vec<String>>, Self::Err> {
		Err(PAErr(Code::Invalid as i32))
	}

	fn source(&self, dev: Option<String>, opts: Self::Cfg, source: crate::cfg::SourceOptions) -> Result<Self::Src, Self::Err> {
		let spec = Spec {
			format: Format::S16NE, // TODO allow more formats?
			channels: source.channels as u8,
			rate: source.sample_rate,
		};
		if !spec.is_valid() {
			return Err(PAErr(0)); // TODO what error number should we throw?
		}
		let attrs = BufferAttr {
			maxlength: opts.server_buffer * source.buffer,
			fragsize: source.buffer,
			..Default::default()
		};
		let simple = Simple::new(
			None,              // Use the default server
			"scope-tui",       // Our application’s name
			Direction::Record, // We want a record stream
			dev.as_deref(),            // Use requested device, or default
			"data",            // Description of our stream
			&spec,             // Our sample format
			None,              // Use default channel map
			Some(&attrs),      // Our hints on how to handle client/server buffers
		)?;
		Ok(PulseSource {
			simple,
			buffer: vec![0; source.buffer as usize],
			channels: source.channels,
		})
	}
}

