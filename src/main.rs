mod app;
mod music;
mod input;
mod display;

use app::App;
use ratatui::{
	backend::CrosstermBackend,
	Terminal,
};
use crossterm::{
	execute, terminal::{
		disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen
	},
};

use clap::{Parser, Subcommand};

use crate::music::Note;

const HELP_TEMPLATE : &str = "{before-help}\
{name} {version} -- by {author}
{about}

{usage-heading} {usage}

{all-args}{after-help}
";

/// a simple oscilloscope/vectorscope for your terminal
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None, help_template = HELP_TEMPLATE)]
pub struct ScopeArgs {
	#[clap(subcommand)]
	source: ScopeSource,

	/// number of channels to open
	#[arg(long, value_name = "N", default_value_t = 2)]
	channels: usize,

	/// tune buffer size to be in tune with given note (overrides buffer option)
	#[arg(long, value_name = "NOTE")]
	tune: Option<String>,

	/// size of audio buffer, and width of scope
	#[arg(short, long, value_name = "SIZE", default_value_t = 2048)]
	buffer: u32,

	/// sample rate to use
	#[arg(long, value_name = "HZ", default_value_t = 48000)]
	sample_rate: u32,

	/// floating point vertical scale, from 0 to 1
	#[arg(short, long, value_name = "x", default_value_t = 1.0)]
	scale: f32,

	/// use vintage looking scatter mode instead of line mode
	#[arg(long, default_value_t = false)]
	scatter: bool,

	/// don't draw reference line
	#[arg(long, default_value_t = false)]
	no_reference: bool,

	/// hide UI and only draw waveforms
	#[arg(long, default_value_t = false)]
	no_ui: bool,

	/// don't use braille dots for drawing lines
	#[arg(long, default_value_t = false)]
	no_braille: bool,
}

#[derive(Debug, Clone, Subcommand)]
pub enum ScopeSource {

	#[cfg(feature = "pulseaudio")]
	/// use PulseAudio Simple api to read data from an audio sink
	Pulse {
		/// source device to attach to
		device: Option<String>,

		/// PulseAudio server buffer size, in block number
		#[arg(long, value_name = "N", default_value_t = 32)]
		server_buffer: u32,
	},

	/// use a file from filesystem and read its content
	File {
		/// path on filesystem of file or pipe
		path: String,

		/// limit data flow to match requested sample rate (UNIMPLEMENTED)
		#[arg(short, long, default_value_t = false)]
		limit_rate: bool,
	},

	/// use new experimental CPAL backend
	Audio {
		/// source device to attach to
		device: Option<String>,

		/// timeout (in seconds) waiting for audio stream
		#[arg(long, default_value_t = 60)]
		timeout: u64,
	}
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let mut args = ScopeArgs::parse();

	if let Some(txt) = &args.tune { // TODO make it less jank
		if let Ok(note) = txt.parse::<Note>() {
			args.buffer = note.tune_buffer_size(args.sample_rate);
			while args.buffer % (args.channels as u32 * 2) != 0 { // TODO customizable bit depth
				args.buffer += 1; // TODO jank but otherwise it doesn't align
			}
		} else {
			eprintln!("[!] Unrecognized note '{}', ignoring option", txt);
		}
	}

	let source = match &args.source {

		#[cfg(feature = "pulseaudio")]
		ScopeSource::Pulse { device, server_buffer } => {
			input::pulse::PulseAudioSimpleDataSource::new(
				device.as_deref(),
				args.channels as u8,
				args.sample_rate,
				args.buffer,
				*server_buffer,
			)?
		},

		ScopeSource::File { path, limit_rate } => {
			input::file::FileSource::new(
				path,
				args.channels,
				args.sample_rate as usize,
				args.buffer as usize,
				*limit_rate
			)?
		},

		ScopeSource::Audio { device, timeout } => {
			input::cpal::DefaultAudioDeviceWithCPAL::new(
				device.as_deref(),
				args.channels as u32,
				args.sample_rate,
				args.buffer,
				*timeout,
			)?
		}
	};

	let mut app = App::from(&args);

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
