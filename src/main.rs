mod parser;
mod app;
mod music;
mod source;
mod display;

use app::App;
use source::PulseAudioSimple;
use ratatui::{
	backend::CrosstermBackend,
	Terminal,
};
use crossterm::{
	execute, terminal::{
		disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen
	},
};

use clap::Parser;

use crate::music::Note;

const HELP_TEMPLATE : &str = "{before-help}\
{name} {version} -- by {author}
{about}

{usage-heading} {usage}

{all-args}{after-help}
";

/// A simple oscilloscope/vectorscope for your terminal
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None, help_template = HELP_TEMPLATE)]
pub struct Args {
	/// Audio device to attach to
	device: Option<String>,

	/// Size of audio buffer, and width of scope
	#[arg(short, long, value_name = "SIZE", default_value_t = 8192)]
	buffer: u32,

	/// Max value, positive and negative, on amplitude scale
	#[arg(short, long, value_name = "SIZE", default_value_t = 20000)]
	range: u32, // TODO counterintuitive, improve this

	/// Use vintage looking scatter mode instead of line mode
	#[arg(long, default_value_t = false)]
	scatter: bool,

	/// Combine left and right channels into vectorscope view
	#[arg(long, default_value_t = false)]
	vectorscope: bool,

	/// Show peaks for each channel as dots
	#[arg(long, default_value_t = true)]
	show_peaks: bool,

	/// Tune buffer size to be in tune with given note (overrides buffer option)
	#[arg(long, value_name = "NOTE")]
	tune: Option<String>,

	/// Number of channels to open
	#[arg(long, value_name = "N", default_value_t = 2)]
	channels: u8,

	/// Sample rate to use
	#[arg(long, value_name = "HZ", default_value_t = 44100)]
	sample_rate: u32,

	/// Pulseaudio server buffer size, in block number
	#[arg(long, value_name = "N", default_value_t = 32)]
	server_buffer: u32,

	/// Start drawing at first rising edge
	#[arg(long, default_value_t = false)]
	triggering: bool,

	/// Threshold value for triggering
	#[arg(long, value_name = "VAL", default_value_t = 0.0)]
	threshold: f64,

	/// Length of trigger check in samples
	#[arg(long, value_name = "SMPL", default_value_t = 1)]
	check_depth: u32,

	/// Trigger upon falling edge instead of rising
	#[arg(long, default_value_t = false)]
	falling_edge: bool,

	/// Don't draw reference line
	#[arg(long, default_value_t = false)]
	no_reference: bool,

	/// Hide UI and only draw waveforms
	#[arg(long, default_value_t = false)]
	no_ui: bool,

	/// Don't use braille dots for drawing lines
	#[arg(long, default_value_t = false)]
	no_braille: bool,
}

fn main() -> Result<(), std::io::Error> {
	let mut args = Args::parse();

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

	let source = PulseAudioSimple::new(
		args.device.as_deref(),
		args.channels,
		args.sample_rate,
		args.buffer,
		args.server_buffer
	).unwrap();

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

	match res {
		Ok(()) => Ok(()),
		Err(e) => {
			eprintln!("[!] Error executing app: {:?}", e);
			Err(e)
		}
	}
}
