use clap::{Parser, Subcommand};
use ratatui::style::Color;

use crate::music::Note;

// TODO is this still necessary?
const HELP_TEMPLATE: &str = "{before-help}\
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
	pub source: ScopeSource,

	#[command(flatten)]
	pub opts: SourceOptions,

	#[command(flatten)]
	pub ui: UiOptions,
}

#[derive(Debug, Clone, Parser)]
pub struct UiOptions {
	/// floating point vertical scale, from 0 to 1
	#[arg(short, long, value_name = "x", default_value_t = 1.0)]
	pub scale: f32,

	/// use vintage looking scatter mode instead of line mode
	#[arg(long, default_value_t = false)]
	pub scatter: bool,

	/// don't draw reference line
	#[arg(long, default_value_t = false)]
	pub no_reference: bool,

	/// hide UI and only draw waveforms
	#[arg(long, default_value_t = false)]
	pub no_ui: bool,

	/// don't use braille dots for drawing lines
	#[arg(long, default_value_t = false)]
	pub no_braille: bool,

	/// palette to use for scope signal lines
	#[arg(long, value_name = "color1,color2", value_delimiter = ',', default_value = "red,yellow,green,magenta")]
	pub palette_color: Vec<Color>,

	/// color to use for axis labels
	#[arg(long, value_name = "color", default_value_t = Color::Cyan)]
	pub labels_color: Color,

	/// color to use for axis lines
	#[arg(long, value_name = "color", default_value_t = Color::DarkGray)]
	pub axis_color: Color,
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

	#[cfg(feature = "file")]
	/// use a file from filesystem and read its content
	File {
		/// path on filesystem of file or pipe
		path: String,

		/// limit data flow to match requested sample rate (UNIMPLEMENTED)
		#[arg(short, long, default_value_t = false)]
		limit_rate: bool,
	},

	#[cfg(feature = "cpal")]
	/// use new experimental CPAL backend
	Audio {
		/// source device to attach to
		device: Option<String>,

		/// timeout (in seconds) waiting for audio stream
		#[arg(long, default_value_t = 5)]
		timeout: u64,

		/// just list available devices and quit
		#[arg(long, default_value_t = false)]
		list: bool,
	},
}

#[derive(Debug, Clone, Parser)]
pub struct SourceOptions {
	/// number of channels to open
	#[arg(short, long, value_name = "N", default_value_t = 2)]
	pub channels: usize,

	/// size of audio buffer, and width of scope
	#[arg(short, long, value_name = "SIZE", default_value_t = 2048)]
	pub buffer: u32,

	/// sample rate to use
	#[arg(short = 'r', long, value_name = "HZ", default_value_t = 48000)]
	pub sample_rate: u32,

	/// tune buffer size to be in tune with given note (overrides buffer option)
	#[arg(short, long, value_name = "NOTE")]
	pub tune: Option<String>,
}

// TODO its convenient to keep this here but it's not really the best place...
impl SourceOptions {
	pub fn tune(&mut self) {
		if let Some(txt) = &self.tune {
			// TODO make it less jank
			if let Ok(note) = txt.parse::<Note>() {
				self.buffer = note.tune_buffer_size(self.sample_rate);
				while self.buffer.is_multiple_of(self.channels as u32 * 2) {
					// TODO customizable bit depth
					self.buffer += 1; // TODO jank but otherwise it doesn't align
				}
			} else {
				eprintln!("[!] Unrecognized note '{}', ignoring option", txt);
			}
		}
	}
}
