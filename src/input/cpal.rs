use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::mpsc;

use super::{stream_to_matrix, Matrix};

pub struct DefaultAudioDeviceWithCPAL {
	rx: mpsc::Receiver<Matrix<f64>>,
	#[allow(unused)]
	stream: cpal::Stream,
	timeout: std::time::Duration,
}

impl DefaultAudioDeviceWithCPAL {
	pub fn instantiate(
		device: Option<&str>,
		opts: &crate::cfg::SourceOptions,
		timeout_secs: u64,
	) -> Result<Box<impl super::DataSource<f64>>, cpal::Error> {
		let host = cpal::default_host();
		let device = match device {
			Some(name) => host
				.input_devices()?
				.find(|x| x.description().map(|d| d.name() == name).unwrap_or(false))
				.ok_or(cpal::Error::new(cpal::ErrorKind::DeviceNotAvailable))?,
			None => host
				.default_input_device()
				.ok_or(cpal::Error::new(cpal::ErrorKind::DeviceNotAvailable))?,
		};

		let max_channels = device
			.supported_input_configs()?
			.map(|x| x.channels())
			.max()
			.unwrap_or(opts.channels as u16);

		let actual_channels = std::cmp::min(opts.channels as u16, max_channels);

		let cfg = cpal::StreamConfig {
			channels: actual_channels,
			buffer_size: cpal::BufferSize::Fixed(opts.buffer),
			sample_rate: opts.sample_rate,
		};
		let (tx, rx) = mpsc::channel();
		let timeout = std::time::Duration::from_secs(timeout_secs);
		let stream = device.build_input_stream(
			cfg,
			move |data: &[f32], _info| {
				tx.send(stream_to_matrix(
					data.iter().cloned(),
					actual_channels as usize,
					1.,
				))
				.unwrap_or(())
			},
			|e| eprintln!("error in input stream: {e}"),
			Some(timeout),
		)?;
		stream.play()?;

		Ok(Box::new(DefaultAudioDeviceWithCPAL { stream, rx, timeout }))
	}
}

impl super::DataSource<f64> for DefaultAudioDeviceWithCPAL {
	fn recv(&mut self) -> Option<super::Matrix<f64>> {
		match self.rx.recv_timeout(self.timeout) {
			Ok(x) => Some(x),
			Err(e) => {
				println!("error receiving from source? {e}");
				None
			}
		}
	}
}
