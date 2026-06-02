use libpulse_binding::{
	def::BufferAttr,
	error::PAErr,
	sample::{Format, Spec},
	stream::Direction,
};
use libpulse_simple_binding::Simple;

use super::{
	format::{SampleParser, Signed16PCM},
	stream_to_matrix,
};

pub struct PulseAudioSimpleDataSource {
	simple: Simple,
	buffer: Vec<u8>,
	channels: usize,
}

impl PulseAudioSimpleDataSource {
	#[allow(clippy::new_ret_no_self)]
	pub fn new(
		device: Option<&str>,
		opts: &crate::cfg::SourceOptions,
		server_buffer: u32,
	) -> Result<Box<dyn super::DataSource<f64>>, PAErr> {
		let sample_size = Signed16PCM::STATIC_SIZE.unwrap_or(4);
		let spec = Spec {
			format: Format::S16NE,	//	TODO: this should probably be mapped to the Signed16PCM format somehow
			channels: opts.channels as u8,
			rate: opts.sample_rate,
		};
		if !spec.is_valid() {
			return Err(PAErr(0)); // TODO what error number should we throw?
		}
		let attrs = BufferAttr {
			maxlength: server_buffer * opts.buffer * opts.channels * sample_size as u32,
			fragsize: opts.buffer,
			..Default::default()
		};
		let simple = Simple::new(
			None,              // Use the default server
			"scope-tui",       // Our application’s name
			Direction::Record, // We want a record stream
			device,            // Use requested device, or default
			"data",            // Description of our stream
			&spec,             // Our sample format
			None,              // Use default channel map
			Some(&attrs),      // Our hints on how to handle client/server buffers
		)?;
		Ok(Box::new(Self {
			simple,
			buffer: vec![0; opts.buffer as usize * opts.channels * sample_size],
			channels: opts.channels,
		}))
	}
}

impl super::DataSource<f64> for PulseAudioSimpleDataSource {
	fn recv(&mut self) -> Option<super::Matrix<f64>> {
		match self.simple.read(&mut self.buffer) {
			Ok(()) => Some(stream_to_matrix(
				Signed16PCM::parse(self.buffer.iter().copied()),
				self.channels,
			)),
			Err(e) => {
				eprintln!("[!] could not receive from pulseaudio: {}", e);
				None
			}
		}
	}
}
