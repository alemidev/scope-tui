mod parser;
mod app;
mod music;

use std::{io::{self, ErrorKind}, time::{Duration, Instant}};
use tui::{
	backend::{CrosstermBackend, Backend},
	widgets::{Block, Chart, Axis, Dataset, GraphType},
	// layout::{Layout, Constraint, Direction},
	Terminal, text::Span, style::{Style, Color, Modifier}, symbols
};
use crossterm::{
	event::{self, DisableMouseCapture, Event, KeyCode, KeyModifiers},
	execute,
	terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use libpulse_simple_binding::Simple;
use libpulse_binding::{stream::Direction, def::BufferAttr};
use libpulse_binding::sample::{Spec, Format};

use clap::Parser;

use parser::{SampleParser, Signed16PCM};

use crate::app::App;
use crate::music::Note;

/// A simple oscilloscope/vectorscope for your terminal
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
	/// Audio device to attach to
	device: Option<String>,

	/// Size of audio buffer, and width of scope
	#[arg(short, long, default_value_t = 8192)]
	buffer: u32,

	/// Max value, positive and negative, on amplitude scale
	#[arg(short, long, default_value_t = 20000)]
	range: u32,

	/// Use vintage looking scatter mode instead of line mode
	#[arg(long, default_value_t = false)]
	scatter: bool,

	/// Combine left and right channels into vectorscope view
	#[arg(long, default_value_t = false)]
	vectorscope: bool,

	/// Tune buffer size to be in tune with given note (overrides buffer option)
	#[arg(long)]
	tune: Option<Note>,

	/// Sample rate to use
	#[arg(long, default_value_t = 44100)]
	sample_rate: u32,

	/// Pulseaudio server buffer size, in block number
	#[arg(long, default_value_t = 32)]
	server_buffer: u32,

	/// Don't draw reference line
	#[arg(long, default_value_t = false)]
	no_reference: bool,

	/// Don't use braille dots for drawing lines
	#[arg(long, default_value_t = false)]
	no_braille: bool,
}

fn poll_event() -> Result<Option<Event>, std::io::Error> {
	if event::poll(Duration::from_millis(0))? {
		Ok(Some(event::read()?))
	} else {
		Ok(None)
	}
}

fn data_set<'a>(
		name: &'a str,
		data: &'a [(f64, f64)],
		marker_type: symbols::Marker,
		graph_type: GraphType,
		axis_color: Color
) -> Dataset<'a> {
	Dataset::default()
		.name(name)
		.marker(marker_type)
		.graph_type(graph_type)
		.style(Style::default().fg(axis_color))
		.data(&data)
}

fn main() -> Result<(), io::Error> {
	let mut args = Args::parse();

	if let Some(note) = &args.tune { // TODO make it less jank
		if note != &Note::INVALID {
			args.buffer = note.tune_buffer_size(0, args.sample_rate);
			while args.buffer % 4 != 0 {
				args.buffer += 1; // TODO jank but otherwise it doesn't align
			}
		}
	}

	// setup terminal
	enable_raw_mode()?;
	let mut stdout = io::stdout();
	execute!(stdout, EnterAlternateScreen)?;
	let backend = CrosstermBackend::new(stdout);
	let mut terminal = Terminal::new(backend)?;
	terminal.hide_cursor()?;

	match run_app(args, &mut terminal) {
		Ok(()) => {},
		Err(e) => {
			println!("[!] Error executing app: {:?}", e);
		}
	}

	// restore terminal
	disable_raw_mode()?;
	execute!(
		terminal.backend_mut(),
		LeaveAlternateScreen,
		DisableMouseCapture
	)?;
	terminal.show_cursor()?;

	Ok(())
}

fn run_app<T : Backend>(args: Args, terminal: &mut Terminal<T>) -> Result<(), io::Error> {
	// prepare globals
	let mut buffer : Vec<u8> = vec![0; args.buffer as usize];
	let mut app = App::from(&args);
	let fmt = Signed16PCM{}; // TODO some way to choose this?

	let mut pause = false;

	// setup audio capture
	let spec = Spec {
		format: Format::S16NE,
		channels: 2,
		rate: args.sample_rate,
	};
	assert!(spec.is_valid());

	let dev = match &args.device {
		Some(d) => Some(d.as_str()),
		None => None,
	};

	let s = match Simple::new(
		None,                // Use the default server
		"ScopeTUI",          // Our application’s name
		Direction::Record,   // We want a record stream
		dev,                 // Use requested device, or default
		"Music",             // Description of our stream
		&spec,               // Our sample format
		None,                // Use default channel map
		Some(&BufferAttr {
			maxlength: args.server_buffer * args.buffer,
			fragsize: args.buffer,
			..Default::default()
		}),
	) {
		Ok(s) => s,
		Err(e) => {
			println!("[!] Could not connect to pulseaudio : {:?}", e);
			return Err(io::Error::new(ErrorKind::Other, "could not connect to pulseaudio"));
		},
	};

	let mut fps = 0;
	let mut framerate = 0;
	let mut last_poll = Instant::now();
	let (mut left, mut right) = (vec![], vec![]);
	let mut merged = vec![];

	loop {
		match s.read(&mut buffer) {
			Ok(()) => {},
			Err(e) => {
				println!("[!] Could not read data from pulseaudio : {:?}", e);
				return Err(io::Error::new(ErrorKind::Other, "could not read from pulseaudio"));
			},
		}

		let mut datasets = vec![];

		if !pause {
			if app.vectorscope() {
				merged = fmt.vectorscope(&mut buffer);
			} else {
				(left, right) = fmt.oscilloscope(&mut buffer);
			}
		}

		if app.cfg.references {
			// for reference in app.references() {
			// 	datasets.push(reference);
			// }
			datasets.push(data_set("", &app.references.x, app.cfg.marker_type, GraphType::Line, app.cfg.axis_color));
			datasets.push(data_set("", &app.references.y, app.cfg.marker_type, GraphType::Line, app.cfg.axis_color));
		}

		if app.vectorscope() {
			let pivot = merged.len() / 2;
			datasets.push(data_set("1", &merged[..pivot], app.cfg.marker_type, app.graph_type(), app.cfg.secondary_color));
			datasets.push(data_set("2", &merged[pivot..], app.cfg.marker_type, app.graph_type(), app.cfg.primary_color));
		} else {
			datasets.push(data_set("R", &right, app.cfg.marker_type, app.graph_type(), app.cfg.secondary_color));
			datasets.push(data_set("L", &left,  app.cfg.marker_type, app.graph_type(), app.cfg.primary_color));
		}
		

		fps += 1;

		if last_poll.elapsed().as_secs() >= 1 {
			framerate = fps;
			fps = 0;
			last_poll = Instant::now();
		}

		terminal.draw(|f| {
			let size = f.size();
			let chart = Chart::new(datasets)
				.block(Block::default().title(
					Span::styled(
						format!(
							"TUI {}  <me@alemi.dev>  --  {} mode  --  range  {}  --  {} samples  --  {:.1} kHz  --  {} fps",
							if app.vectorscope() { "Vectorscope" } else { "Oscilloscope" },
							if app.scatter() { "scatter" } else { "line" },
							app.scale(), app.width(), args.sample_rate as f32 / 1000.0, framerate,
						),
					Style::default().add_modifier(Modifier::BOLD).fg(Color::Yellow))
				))
				.x_axis(Axis::default()
					.title(Span::styled(app.name(app::Axis::X), Style::default().fg(Color::Cyan)))
					.style(Style::default().fg(app.cfg.axis_color))
					.bounds(app.bounds(app::Axis::X))) // TODO allow to have axis sometimes?
				.y_axis(Axis::default()
					.title(Span::styled(app.name(app::Axis::Y), Style::default().fg(Color::Cyan)))
					.style(Style::default().fg(app.cfg.axis_color))
					.bounds(app.bounds(app::Axis::Y)));
			f.render_widget(chart, size)
		})?;

		if let Some(Event::Key(key)) = poll_event()? {
			match key.modifiers {
				KeyModifiers::CONTROL => {
					match key.code {
						KeyCode::Char('c') => break,
						_ => {},
					}
				},
				_ => {
					match key.code {
						KeyCode::Char('q') => break,
						KeyCode::Char(' ') => pause = !pause,
						KeyCode::Char('=') => app.update_scale(-1000),
						KeyCode::Char('-') => app.update_scale(1000),
						KeyCode::Char('+') => app.update_scale(-100),
						KeyCode::Char('_') => app.update_scale(100),
						KeyCode::Char('v') => app.set_vectorscope(!app.vectorscope()),
						KeyCode::Char('s') => app.set_scatter(!app.scatter()),
						_ => {},
					}
				}
			}
		}
	}

	Ok(())
}
