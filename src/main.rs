mod app;
mod cfg;
mod music;
mod input;
mod display;

use app::App;
use cfg::{ScopeArgs, ScopeSource};
use clap::Parser;
use ratatui::{backend::CrosstermBackend, Terminal};
use crossterm::{execute, terminal::{
	disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen
}};


fn main() -> Result<(), Box<dyn std::error::Error>> {
	let mut args = ScopeArgs::parse();
	args.opts.tune();

	let source = match args.source {
		#[cfg(feature = "pulseaudio")]
		ScopeSource::Pulse { device, server_buffer } => {
			input::pulse::PulseAudioSimpleDataSource::new(device.as_deref(), &args.opts, server_buffer)?
		},

		#[cfg(feature = "file")]
		ScopeSource::File { path, limit_rate } => {
			input::file::FileSource::new(&path, &args.opts, limit_rate)?
		},

		#[cfg(feature = "cpal")]
		ScopeSource::Audio { device, timeout } => {
			input::cpal::DefaultAudioDeviceWithCPAL::new(device.as_deref(), &args.opts, timeout)?
		}
	};

	let mut app = App::new(&args.ui, &args.opts);

	// setup terminal
	enable_raw_mode()?;
	let mut stdout = std::io::stdout();
	execute!(stdout, EnterAlternateScreen)?;
	let backend = CrosstermBackend::new(stdout);
	let mut terminal = Terminal::new(backend)?;
	terminal.hide_cursor()?;

	let res = app.run(source, &mut terminal);

	// restore terminal
	disable_raw_mode()?;
	execute!(
		terminal.backend_mut(),
		LeaveAlternateScreen,
	)?;
	terminal.show_cursor()?;

	if let Err(e) = res {
		eprintln!("[!] Error executing app: {:?}", e);
	}

	Ok(())
}
