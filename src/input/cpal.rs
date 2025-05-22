use std::sync::mpsc;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use super::{stream_to_matrix, Matrix};

pub struct DefaultAudioDeviceWithCPAL {
	rx: mpsc::Receiver<Matrix<f64>>,
	#[allow(unused)]
	stream: cpal::Stream,
}

#[derive(Debug, thiserror::Error)]
pub enum AudioDeviceErrors {
	#[error("{0}")]
	Device(#[from] cpal::DevicesError),

	#[error("device not found")]
	NotFound,

	#[error("{0}")]
	BuildStream(#[from] cpal::BuildStreamError),

	#[error("{0}")]
	PlayStream(#[from] cpal::PlayStreamError),

	#[error("{0}")]
	SupportedStreamsError(#[from] cpal::SupportedStreamConfigsError),
}

impl DefaultAudioDeviceWithCPAL {
	pub fn new(device: Option<&str>, opts: &crate::cfg::SourceOptions, timeout_secs: u64) -> Result<Box<impl super::DataSource<f64>>, AudioDeviceErrors> {
		let host = cpal::default_host();
		let device = match device {
			Some(name) => host
				.input_devices()?
				.find(|x| x.name().as_deref().unwrap_or("") == name)
				.ok_or(AudioDeviceErrors::NotFound)?,
			None => host
				.default_input_device()
				.ok_or(AudioDeviceErrors::NotFound)?,
		};

		let max_channels = device
			.supported_input_configs()?
			.map(|x| x.channels())
			.max()
			.unwrap_or(opts.channels as u16);

		let actual_channels = std::cmp::min(opts.channels as u16, max_channels);

		let cfg = cpal::StreamConfig {
			channels: actual_channels,
			buffer_size: cpal::BufferSize::Fixed(opts.buffer * actual_channels as u32 * 2),
			sample_rate: cpal::SampleRate(opts.sample_rate),
		};
		let (tx, rx) = mpsc::channel();
		let stream = device.build_input_stream(
			&cfg,
			move |data:&[f32], _info| {
				tx.send(
					stream_to_matrix(data.iter().cloned(), actual_channels as usize, 1.)
				)
					.unwrap_or(())
			},
			|e| eprintln!("error in input stream: {e}"),
			Some(std::time::Duration::from_secs(timeout_secs)),
		)?;
		stream.play()?;

		Ok(Box::new(DefaultAudioDeviceWithCPAL { stream, rx }))
	}
}

impl super::DataSource<f64> for DefaultAudioDeviceWithCPAL {
	fn recv(&mut self) -> Option<super::Matrix<f64>> {
		match self.rx.recv() {
			Ok(x) => Some(x),
			Err(e) => {
				println!("error receiving from source? {e}");
				None
			},
		}
	}
}
