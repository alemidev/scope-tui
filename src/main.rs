mod parser;
mod app;
mod music;

use std::{io::{self, ErrorKind}, time::{Duration, Instant}};
use tui::{
	backend::{CrosstermBackend, Backend},
	widgets::{Block, Chart, Axis, Dataset, GraphType},
	// layout::{Layout, Constraint, Direction},
	Terminal, text::Span, style::{Style, Color, Modifier}, symbols, layout::Alignment
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

	/// Tune buffer size to be in tune with given note (overrides buffer option)
	#[arg(long, value_name = "NOTE")]
	tune: Option<Note>,

	/// Sample rate to use
	#[arg(long, value_name = "HZ", default_value_t = 44100)]
	sample_rate: u32,

	/// Pulseaudio server buffer size, in block number
	#[arg(long, value_name = "N", default_value_t = 32)]
	server_buffer: u32,

	/// Don't draw reference line
	#[arg(long, default_value_t = false)]
	no_reference: bool,

	/// Don't use braille dots for drawing lines
	#[arg(long, default_value_t = false)]
	no_braille: bool,
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
	let mut channels = vec![];

	loop {
		match s.read(&mut buffer) {
			Ok(()) => {},
			Err(e) => {
				println!("[!] Could not read data from pulseaudio : {:?}", e);
				return Err(io::Error::new(ErrorKind::Other, "could not read from pulseaudio"));
			},
		}

		if !pause {
			channels = fmt.oscilloscope(&mut buffer, 2);
		}

			}
		}

		let mut measures;

		if app.vectorscope() {
			measures = vec![];
			for chunk in channels.chunks(2) {
				let mut tmp = vec![];
				for i in 0..chunk[0].len() {
					tmp.push((chunk[0][i] as f64, chunk[1][i] as f64));
				}
				let pivot = tmp.len() / 2;
				measures.push(tmp[..pivot].to_vec());
				measures.push(tmp[pivot..].to_vec());
			}
		} else {
			measures = vec![vec![]; channels.len()];
			for i in 0..channels[0].len() {
				for j in 0..channels.len() {
					measures[j].push((i as f64, channels[j][i]));
				}
			}
		}

		let mut datasets = vec![];

		if app.cfg.references {
			datasets.push(data_set("", &app.references.x, app.cfg.marker_type, GraphType::Line, app.cfg.axis_color));
			datasets.push(data_set("", &app.references.y, app.cfg.marker_type, GraphType::Line, app.cfg.axis_color));
		}

		let ds_names = if app.vectorscope() { vec!["2", "1"] } else { vec!["R", "L"] };
		let palette : Vec<Color> = app.cfg.palette.iter().rev().map(|x| x.clone()).collect();

		for (i, ds) in measures.iter().rev().enumerate() {
			datasets.push(data_set(ds_names[i], ds, app.cfg.marker_type, app.graph_type(), palette[i % palette.len()]));
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
				.block(block(&app, args.sample_rate as f32, framerate))
				.x_axis(axis(&app, app::Dimension::X)) // TODO allow to have axis sometimes?
				.y_axis(axis(&app, app::Dimension::Y));
			f.render_widget(chart, size)
		})?;

		if let Some(Event::Key(key)) = poll_event()? {
			match key.modifiers {
				KeyModifiers::CONTROL => {
					match key.code {
						KeyCode::Char('c') | KeyCode::Char('q') | KeyCode::Char('w') => break,
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
						KeyCode::Char('h') => app.cfg.references = !app.cfg.references,
						_ => {},
					}
				}
			}
		}
	}

	Ok(())
}


// TODO these functions probably shouldn't be here

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

fn axis(app: &App, dim: app::Dimension) -> Axis {
	let mut a = Axis::default();
	if app.cfg.references {
		a = a.title(Span::styled(app.name(&dim), Style::default().fg(Color::Cyan)));
	}
	a.style(Style::default().fg(app.cfg.axis_color))
		.bounds(app.bounds(&dim))
}

fn block(app: &App, sample_rate: f32, framerate: u32) -> Block {
	let mut b = Block::default();

	if app.cfg.references {
		b = b.title(
			Span::styled(
				format!(
					"TUI {}  --  {} mode  --  range  {}  --  {} samples  --  {:.1} kHz  --  {} fps",
					if app.vectorscope() { "Vectorscope" } else { "Oscilloscope" },
					if app.scatter() { "scatter" } else { "line" },
					app.scale(), app.width(), sample_rate / 1000.0, framerate,
				),
			Style::default().add_modifier(Modifier::BOLD).fg(Color::Yellow))
		).title_alignment(Alignment::Center);
	}

	b
}
