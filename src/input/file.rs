use std::{fs::File, io::Read};

use super::{format::{SampleParser, Signed16PCM}, stream_to_matrix, Matrix};

pub struct FileSource {
	file: File,
	buffer: Vec<u8>,
	channels: usize,
	sample_rate: usize,
	limit_rate: bool,
	// TODO when all data is available (eg, file) limit data flow to make it
	// somehow visualizable. must be optional because named pipes block
	// TODO support more formats
}

impl FileSource {
	#[allow(clippy::new_ret_no_self)]
	pub fn new(path: &str, opts: &crate::cfg::SourceOptions, limit_rate: bool) -> Result<Box<dyn super::DataSource<f64>>, std::io::Error> {
		Ok(Box::new(
			FileSource {
				channels: opts.channels,
				sample_rate: opts.sample_rate as usize,
				limit_rate,
				file: File::open(path)?,
				buffer: vec![0u8; opts.buffer as usize * opts.channels],
			}
		))
	}
}

impl super::DataSource<f64> for FileSource {
	fn recv(&mut self) -> Option<Matrix<f64>> {
		match self.file.read_exact(&mut self.buffer) {
			Ok(()) => Some(
				stream_to_matrix(
					self.buffer.chunks(2).map(Signed16PCM::parse),
					self.channels,
					32768.0,
				)
			),
			Err(_e) => None, // TODO log it
		}
	}
}
