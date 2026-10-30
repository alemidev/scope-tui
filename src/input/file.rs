use std::{
	fs::File,
	io::{self, Read},
};

use super::{
	format::{SampleParser, Signed16PCM},
	stream_to_matrix, SampleTape,
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

impl super::Source for FileSource {
	fn recv(&mut self) -> Option<SampleTape> {
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

#[derive(Default)]
pub struct FileHost;

impl super::Host for FileHost {
	type Err = std::io::Error;
	type Src = FileSource;
	type Cfg = crate::cfg::FileOptions;

	fn header() -> Vec<String> {
		vec!["TITLE".to_string()]
	}

	fn list(&self) -> Result<Vec<Vec<String>>, Self::Err> {
		Ok(
			std::fs::read_dir(std::env::current_dir()?)?
				.filter_map(|d| d.ok())
				.map(|d| vec![d.path().to_string_lossy().to_string()])
				.collect()
		)
	}

	fn source(&self, dev: Option<String>, opts: Self::Cfg, source: crate::cfg::SourceOptions) -> Result<Self::Src, Self::Err> {
		let samples_per_batch = (source.buffer * source.channels as u32) / 2;
		let batches_per_second = source.sample_rate / samples_per_batch;
		let ms_sleep = (1000 / batches_per_second) as u64;
		Ok(FileSource {
			channels: source.channels,
			limit_rate: opts.limit_rate,
			ms_sleep,
			file: File::open(dev.ok_or(std::io::Error::from(std::io::ErrorKind::NotFound))?)?,
			buffer: vec![0u8; source.buffer as usize * source.channels],
		})
	}
}
