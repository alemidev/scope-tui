use clap::Parser;
use scope::app::App;
use scope::cfg::{ScopeArgs, ScopeSource};

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let mut args = ScopeArgs::parse();
	args.opts.tune();

	let source = match args.source {
		#[cfg(feature = "pulseaudio")]
		ScopeSource::Pulse {
			device,
			server_buffer,
		} => scope::input::pulse::PulseAudioSimpleDataSource::new(
			device.as_deref(),
			&args.opts,
			server_buffer,
		)?,

		#[cfg(feature = "file")]
		ScopeSource::File { path, limit_rate } => {
			scope::input::file::FileSource::new(&path, &args.opts, limit_rate)?
		}

		#[cfg(feature = "cpal")]
		ScopeSource::Audio {
			device,
			timeout,
			list,
		} => {
			if list {
				use cpal::traits::{DeviceTrait, HostTrait};
				let host = cpal::default_host();
				for dev in host.input_devices().unwrap() {
					println!("> {}", dev.name().unwrap());
					for config in dev.supported_input_configs().unwrap() {
						let bufsize = match config.buffer_size() {
							cpal::SupportedBufferSize::Range { min, max } => (*min, *max),
							cpal::SupportedBufferSize::Unknown => (0, 0),
						};
						println!(
							"  + {}ch {}-{}hz {}-{}buf ({})",
							config.channels(),
							config.min_sample_rate().0,
							config.max_sample_rate().0,
							bufsize.0,
							bufsize.1,
							config.sample_format()
						);
					}
				}
				return Ok(());
			}
			scope::input::cpal::DefaultAudioDeviceWithCPAL::instantiate(
				device.as_deref(),
				&args.opts,
				timeout,
			)?
		}
	};

	let mut app = App::new(&args.ui, &args.opts);

	// set panic hook to restore the terminal
	let hook = std::panic::take_hook();
	std::panic::set_hook(Box::new(move |panic_info| {
		ratatui::restore();
		hook(panic_info);
	}));

	let term = ratatui::init();
	let res = app.run(source, term);

	ratatui::restore();

	if let Err(e) = res {
		eprintln!("[!] Error executing app: {:?}", e);
	}

	Ok(())
}
