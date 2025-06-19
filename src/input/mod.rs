pub mod format;

#[cfg(feature = "pulseaudio")]
pub mod pulse;

#[cfg(feature = "file")]
pub mod file;

#[cfg(feature = "cpal")]
pub mod cpal;

pub type Matrix<T> = Vec<Vec<T>>;

pub trait DataSource<T> {
	fn recv(&mut self) -> Option<Matrix<T>>; // TODO convert in Result and make generic error
}

/// separate a stream of alternating channels into a matrix of channel streams:
///   L R L R L R L R L R
/// becomes
///   L L L L L
///   R R R R R
pub fn stream_to_matrix<I, O>(
	stream: impl Iterator<Item = I>,
	channels: usize,
	norm: O,
) -> Matrix<O>
where
	I: Copy + Into<O>,
	O: Copy + std::ops::Div<Output = O>,
{
	let mut out = vec![vec![]; channels];
	let mut channel = 0;
	for sample in stream {
		out[channel].push(sample.into() / norm);
		channel = (channel + 1) % channels;
	}
	out
}
