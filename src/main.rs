use clap::Parser;
use scope::app::App;
use scope::cfg::{ScopeArgs, ScopeSource};

fn main() {
	let mut args = ScopeArgs::parse();
	args.opts.tune();

	// set panic hook to restore the terminal in case of sudden interruptions
	let hook = std::panic::take_hook();
	std::panic::set_hook(Box::new(move |panic_info| {
		ratatui::restore();
		hook(panic_info);
	}));

	if args.ui.picker {
		let (header, devices) = match device_list(&args) {
			Ok(d) => d,
			Err(e) => {
				eprintln!("[!] error listing devices: {e} - {e:?}");
				return;
			}
		};
		let mut picker = scope::picker::Picker::new(header, devices);

		let term = ratatui::init();
		args.device = picker.run(term);
		ratatui::restore();

		if args.device.is_none() {
			return;
		}
	}

	let mut app = App::new(&args.ui, &args.opts);

	let source = match source(args) {
		Ok(s) => s,
		Err(e) => {
			eprintln!("[!] error creating source: {e} - {e:?}");
			return;
		}
	};

	let term = ratatui::init();
	let res = app.run(source, term);
	ratatui::restore();

	if let Err(e) = res {
		eprintln!("[!] Error executing app: {:?}", e);
	}
}

fn device_list(args: &ScopeArgs) -> Result<(Vec<String>, Vec<Vec<String>>), scope::input::HostError> {
	use scope::input::Host;
	Ok(match args.source {
		#[cfg(feature = "pulseaudio")]
		ScopeSource::Pulse(ref _opts) => (scope::input::pulse::PulseHost::header(), scope::input::pulse::PulseHost.list()?),
		#[cfg(feature = "file")]
		ScopeSource::File(ref _opts) => (scope::input::file::FileHost::header(), scope::input::file::FileHost.list()?),
		#[cfg(feature = "cpal")]
		ScopeSource::Audio(ref _opts) => (scope::input::audio::CpalHost::header(), scope::input::audio::CpalHost::default().list()?),
	})
}

fn source(args: ScopeArgs) -> Result<Box<dyn scope::input::Source>, scope::input::HostError> {
	use scope::input::Host;
	Ok(match args.source {
		#[cfg(feature = "pulseaudio")]
		ScopeSource::Pulse(opts) => Box::new(scope::input::pulse::PulseHost.source(args.device, opts, args.opts)?),
		#[cfg(feature = "file")]
		ScopeSource::File(opts) => Box::new(scope::input::file::FileHost.source(args.device, opts, args.opts)?),
		#[cfg(feature = "cpal")]
		ScopeSource::Audio(opts) => Box::new(scope::input::audio::CpalHost::default().source(args.device, opts, args.opts)?),
	})
}
