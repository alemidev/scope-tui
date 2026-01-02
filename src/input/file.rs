use std::{
	fs::File,
	io::{self, Read},
};

use super::{
	format::{SampleParser, Signed16PCM},
	stream_to_matrix, Matrix,
};

/// Reads the file into the buffer until the buffer is full or the EOF is reached.
///
/// If it reaches the EOF before the buffer is full pad the buffer with zeros.
pub fn read_with_padding(file: &mut File, buffer: &mut [u8]) -> io::Result<()> {
	let mut read_so_far = 0;

	while read_so_far < buffer.len() {
		let remaining_slice = &mut buffer[read_so_far..];

		let n = file.read(remaining_slice)?;
		if n > 0 {
			read_so_far += n;
		} else {
			// End of File reached -> pad with zeros
			// buffer: [........0000]
			for b in remaining_slice {
				*b = 0;
			}

			return Ok(());
		}
	}

	Ok(())
}

pub struct FileSource {
	file: File,
	buffer: Vec<u8>,
	channels: usize,
	_sample_rate: usize,
	_limit_rate: bool,
	// TODO when all data is available (eg, file) limit data flow to make it
	// somehow visualizable. must be optional because named pipes block
	// TODO support more formats
}

impl FileSource {
	#[allow(clippy::new_ret_no_self)]
	pub fn new(
		path: &str,
		opts: &crate::cfg::SourceOptions,
		limit_rate: bool,
	) -> Result<Box<dyn super::DataSource<f64>>, std::io::Error> {
		Ok(Box::new(FileSource {
			channels: opts.channels,
			_sample_rate: opts.sample_rate as usize,
			_limit_rate: limit_rate,
			file: File::open(path)?,
			buffer: vec![0u8; opts.buffer as usize * opts.channels],
		}))
	}
}

impl super::DataSource<f64> for FileSource {
	fn recv(&mut self) -> Option<Matrix<f64>> {
		match read_with_padding(&mut self.file, &mut self.buffer) {
			Ok(()) => Some(stream_to_matrix(
				self.buffer.chunks(2).map(Signed16PCM::parse),
				self.channels,
				32768.0,
			)),
			Err(_e) => None, // TODO log it
		}
	}
}
