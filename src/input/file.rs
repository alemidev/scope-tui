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
	limit_rate: bool,
	ms_sleep: u64,
	// TODO support more formats
}

impl FileSource {
	#[allow(clippy::new_ret_no_self)]
	pub fn new(
		path: &str,
		opts: &crate::cfg::SourceOptions,
		limit_rate: bool,
	) -> Result<Box<dyn super::DataSource<f64>>, std::io::Error> {
		let samples_per_batch = (opts.buffer * opts.channels as u32) / 2;
		let batches_per_second = opts.sample_rate / samples_per_batch;
		let ms_sleep = (1000 / batches_per_second) as u64;
		Ok(Box::new(FileSource {
			channels: opts.channels,
			limit_rate,
			ms_sleep,
			file: File::open(path)?,
			buffer: vec![0u8; opts.buffer as usize * opts.channels],
		}))
	}
}

impl super::DataSource<f64> for FileSource {
	fn recv(&mut self) -> Option<Matrix<f64>> {
		if self.limit_rate {
			std::thread::sleep(std::time::Duration::from_millis(self.ms_sleep));
		}
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
