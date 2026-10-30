pub mod format;
pub mod tape;

#[cfg(feature = "pulseaudio")]
pub mod pulse;

#[cfg(feature = "file")]
pub mod file;

#[cfg(feature = "cpal")]
pub mod audio;

/// a single sample, arbitrarily chosen to be a long float
pub type Sample = f64;
/// one sample in time, multiple channels
pub type ChannelTape = Vec<Sample>;
/// a tape containing multiple samples over time
pub type SampleTape = Vec<ChannelTape>;


/// an audio host generic interface
pub trait Host {
	/// kind of error this host can throw
	type Err: std::error::Error;
	/// the source type that this host produces
	type Src: Source;
	/// required config to build a new source
	type Cfg;

	/// the titles to put in table header for these sources
	fn header() -> Vec<String>;

	/// lists available devices (sources)
	fn list(&self) -> Result<Vec<Vec<String>>, Self::Err>;

	/// initialize a new audio source
	fn source(&self, dev: Option<String>, cfg: Self::Cfg, opts: crate::cfg::SourceOptions) -> Result<Self::Src, Self::Err>;
}

/// an audio source generic interface
pub trait Source {
	/// receives next chunk of audio data
	fn recv(&mut self) -> Option<SampleTape>; // TODO convert in Result and make generic error
}

/// utility wrapper error to make working with different sources less painful
#[derive(Debug, thiserror::Error)]
pub enum HostError {
	#[cfg(feature = "cpal")]
	#[error("cpal host error: {0:?}")]
	CpalError(#[from] cpal::Error),

	#[cfg(feature = "pulseaudio")]
	#[error("pulse host error: {0:?}")]
	PulseError(#[from] libpulse_binding::error::PAErr),

	#[cfg(feature = "file")]
	#[error("file host error: {0:?}")]
	FileError(#[from] std::io::Error),

}

/// separate a stream of alternating channels into a matrix of channel streams:
///   L R L R L R L R L R
/// becomes
///   L L L L L
///   R R R R R
pub fn stream_to_matrix<I>(
	stream: impl Iterator<Item = I>,
	channels: usize,
	norm: f64,
) -> SampleTape
where
	I: Copy + Into<f64>,
{
	let mut out = vec![vec![]; channels];
	let mut channel = 0;
	for sample in stream {
		out[channel].push(sample.into() / norm);
		channel = (channel + 1) % channels;
	}

	out
}
