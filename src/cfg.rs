use clap::{Parser, Subcommand};
use ratatui::style::Color;

use crate::{app::CurrentDisplayMode, music::Note};

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
	#[command(flatten)]
	pub ui: UiOptions,

	#[clap(subcommand)]
	pub source: ScopeSource,

	#[command(flatten)]
	pub opts: SourceOptions,

	/// device to attach to, by name
	pub device: Option<String>,
}

#[derive(Debug, Clone, Parser)]
pub struct UiOptions {
	/// run device picker before starting scope-tui
	#[arg(short, long, default_value_t = false)]
	pub picker: bool,

	/// limits max framerate if set
	#[arg(long = "fps")]
	pub framerate: Option<u64>,

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
	#[arg(long, value_name = "color1,color2", value_delimiter = ',', default_value = "red,yellow,cyan,magenta")]
	pub palette_color: Vec<Color>,

	/// color to use for axis labels
	#[arg(long, value_name = "color", default_value_t = Color::Cyan)]
	pub labels_color: Color,

	/// color to use for axis lines
	#[arg(long, value_name = "color", default_value_t = Color::DarkGray)]
	pub axis_color: Color,

	/// mode to start in
	#[arg(long, value_enum, default_value_t = CurrentDisplayMode::Oscilloscope)]
	pub mode: CurrentDisplayMode,
}

#[derive(Debug, Clone, Subcommand)]
pub enum ScopeSource {
	#[cfg(feature = "pulseaudio")]
	/// use PulseAudio Simple api to read data from an audio sink
	Pulse(#[clap(flatten)] PulseOptions),

	#[cfg(feature = "file")]
	/// use a file from filesystem and read its content
	File(#[clap(flatten)] FileOptions),

	#[cfg(feature = "cpal")]
	/// use new experimental CPAL backend
	Audio(#[clap(flatten)] CpalOptions),
}

#[cfg(feature = "pulseaudio")]
#[derive(Debug, Clone, Parser)]
pub struct PulseOptions {
	/// PulseAudio server buffer size, in block number
	#[arg(long, value_name = "N", default_value_t = 1)]
	pub server_buffer: u32,
}

#[cfg(feature = "file")]
#[derive(Debug, Clone, Parser)]
pub struct FileOptions {
	/// limit data flow to match requested sample rate
	#[arg(short, long, default_value_t = false)]
	pub limit_rate: bool,
}

#[cfg(feature = "cpal")]
#[derive(Debug, Clone, Parser)]
pub struct CpalOptions {
	/// timeout (in seconds) waiting for audio stream
	#[arg(long, default_value_t = 5)]
	pub timeout: u64,
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

	/// number of tape chunks to keep in memory (for smooth scrolling)
	#[arg(long, default_value_t = 32)]
	pub chunks: usize,
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
