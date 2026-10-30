use cpal::{SampleFormat, SupportedBufferSize, traits::{DeviceTrait, HostTrait, StreamTrait}};
use std::sync::mpsc;

use super::{stream_to_matrix, SampleTape};

pub struct CpalSource {
	rx: mpsc::Receiver<SampleTape>,
	#[allow(unused)]
	stream: cpal::Stream,
	timeout: std::time::Duration,
}

impl super::Source for CpalSource {
	fn recv(&mut self) -> Option<super::SampleTape> {
		match self.rx.recv_timeout(self.timeout) {
			Ok(x) => Some(x),
			Err(e) => {
				println!("error receiving from source? {e}");
				None
			}
		}
	}
}

#[derive(Default)]
pub struct CpalHost {
	host: cpal::Host,
}

impl super::Host for CpalHost {
	type Err = cpal::Error;
	type Src = CpalSource;
	type Cfg = crate::cfg::CpalOptions;

	fn header() -> Vec<String> {
		vec![
			"NAME".to_string(),
			"CHANNELS".to_string(),
			"BUFFER".to_string(),
			"DIRECTION".to_string(),
			"DRIVER".to_string(),
			"FORMATS".to_string(),
		]
	}

	fn list(&self) -> Result<Vec<Vec<String>>, cpal::Error> {
		Ok(
			self.host.input_devices()?
				.filter_map(|d| Some((d.description().ok()?, d.supported_input_configs().ok()?)))
				.map(|(d, cfgs)| {
					let mut formats = Vec::new();
					let mut channels = MinMax::new();
					let mut bufsizes = MinMax::new();
					for cfg in cfgs {
						if matches!(cfg.sample_format(), SampleFormat::F32 | SampleFormat::F64 | SampleFormat::I16) {
							formats.push(cfg.sample_format().to_string());
						}
						channels.push(cfg.channels() as usize);
						if let SupportedBufferSize::Range { min, max } = cfg.buffer_size() {
							bufsizes.push_min(*min as usize);
							bufsizes.push_max(*max as usize);
						}
					}

					vec![
						d.name().to_string(),
						format!("{}..={}", channels.min, channels.max),
						format!("{}..={}", bufsizes.min, bufsizes.max),
						d.direction().to_string(),
						d.driver().unwrap_or_default().to_string(),
						formats.join(","),
						
					]
				})
				.collect()
		)
	}

	fn source(&self, dev: Option<String>, opts: Self::Cfg, source: crate::cfg::SourceOptions) -> Result<Self::Src, Self::Err> {
		let device = match dev {
			Some(name) => self.host
				.input_devices()?
				.find(|x| x.description().map(|d| d.name() == name).unwrap_or(false))
				.ok_or(cpal::Error::new(cpal::ErrorKind::DeviceNotAvailable))?,
			None => self.host
				.default_input_device()
				.ok_or(cpal::Error::new(cpal::ErrorKind::DeviceNotAvailable))?,
		};

		let max_channels = device
			.supported_input_configs()?
			.map(|x| x.channels())
			.max()
			.unwrap_or(source.channels as u16);

		let actual_channels = std::cmp::min(source.channels as u16, max_channels);

		let cfg = cpal::StreamConfig {
			channels: actual_channels,
			buffer_size: cpal::BufferSize::Fixed(source.buffer),
			sample_rate: source.sample_rate,
		};
		let (tx, rx) = mpsc::channel();
		let timeout = std::time::Duration::from_secs(opts.timeout);
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

		Ok(CpalSource { stream, rx, timeout })
	}
}

struct MinMax {
	min: usize,
	max: usize,
}

impl MinMax {
	fn new() -> Self {
		Self { min: usize::MAX, max: usize::MIN }
	}

	fn push_min(&mut self, v: usize) {
		if v < self.min {
			self.min = v;
		}
	}
	fn push_max(&mut self, v: usize) {
		if v > self.max {
			self.max = v;
		}
	}

	fn push(&mut self, v: usize) {
		self.push_min(v);
		self.push_max(v);
	}
}
