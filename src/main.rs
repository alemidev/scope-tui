mod parser;
mod app;

use std::{io, time::Duration};
use tui::{
	backend::CrosstermBackend,
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
use app::AppConfig;

/// A simple oscilloscope/vectorscope for your terminal
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
	/// Audio device to attach to
	device: Option<String>,

	/// Size of audio buffer, and width of scope
	#[arg(short, long, default_value_t = 8192)]
	width: u32,

	/// Max value on Amplitude scale
	#[arg(short, long, default_value_t = 20000)]
	scale: u32,

	/// Don't draw reference line
	#[arg(long, default_value_t = false)]
	no_reference: bool,

	/// Don't use braille dots for drawing lines
	#[arg(long, default_value_t = false)]
	no_braille: bool,

	/// Use vintage looking scatter mode
	#[arg(long, default_value_t = false)]
	scatter: bool,

	/// Combine left and right channels into vectorscope view
	#[arg(long, default_value_t = false)]
	vectorscope: bool,
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
		data: &'a Vec<(f64, f64)>,
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
	let args = Args::parse();

	// setup audio capture
	let spec = Spec {
		format: Format::S16NE,
		channels: 2,
		rate: 44100,
	};
	assert!(spec.is_valid());

	let dev = match &args.device {
		Some(d) => Some(d.as_str()),
		None => None,
	};


	let s = Simple::new(
		None,                // Use the default server
		"ScopeTUI",          // Our application’s name
		Direction::Record,   // We want a record stream
		dev,                 // Use requested device, or default
		"Music",             // Description of our stream
		&spec,               // Our sample format
		None,                // Use default channel map
		Some(&BufferAttr {
			maxlength: 32 * args.width,
			fragsize: args.width,
			..Default::default()
		}),
	).unwrap();

	// setup terminal
	enable_raw_mode()?;
	let mut stdout = io::stdout();
	execute!(stdout, EnterAlternateScreen)?;
	let backend = CrosstermBackend::new(stdout);
	let mut terminal = Terminal::new(backend)?;
	terminal.hide_cursor().unwrap();

	// prepare globals
	let mut buffer : Vec<u8> = vec![0; args.width as usize];
	let mut cfg = AppConfig::from(args);
	let fmt = Signed16PCM{}; // TODO some way to choose this?

	let mut pause = false;

	loop {
		s.read(&mut buffer).unwrap();

		if !pause {
			let mut datasets = vec![];
			let (left, right);
			let merged;

			let mut ref_data_x = Vec::new();
			let mut ref_data_y = Vec::new();

			if cfg.references {
		
				if cfg.vectorscope() {
					for x in -(cfg.scale() as i64)..(cfg.scale() as i64) {
						ref_data_x.push((x as f64, 0 as f64));
						ref_data_y.push((0 as f64, x as f64));
					}
				} else {
					for x in 0..cfg.width() {
						ref_data_x.push((x as f64, 0 as f64));
					}
					for y in -(cfg.scale() as i64)..(cfg.scale() as i64) {
						ref_data_y.push(((cfg.width() as f64) / 2.0, y as f64));
					}
				}
		
				datasets.push(data_set("X", &ref_data_x, cfg.marker_type, GraphType::Line, cfg.axis_color));
				datasets.push(data_set("Y", &ref_data_y, cfg.marker_type, GraphType::Line, cfg.axis_color));
			}

			if cfg.vectorscope() {
				merged = fmt.vectorscope(&mut buffer);
				datasets.push(data_set("V", &merged, cfg.marker_type, cfg.graph_type(), cfg.primary_color));
			} else {
				(left, right) = fmt.oscilloscope(&mut buffer);
				datasets.push(data_set("R", &right, cfg.marker_type, cfg.graph_type(), cfg.secondary_color));
				datasets.push(data_set("L", &left, cfg.marker_type, cfg.graph_type(), cfg.primary_color));
			}

			terminal.draw(|f| {
				let size = f.size();
				let chart = Chart::new(datasets)
					.block(Block::default().title(
						Span::styled(
							format!(
								"TUI {}  <me@alemi.dev>  --  {} mode  --  range  {}  --  {} samples",
								if cfg.vectorscope() { "Vectorscope" } else { "Oscilloscope" },
								if cfg.scatter() { "scatter" } else { "line" },
								cfg.scale(), cfg.width(),
							),
						Style::default().add_modifier(Modifier::BOLD).fg(Color::Yellow))
					))
					.x_axis(Axis::default()
						.title(Span::styled(cfg.name(app::Axis::X), Style::default().fg(Color::Cyan)))
						.style(Style::default().fg(cfg.axis_color))
						.bounds(cfg.bounds(app::Axis::X)))
					.y_axis(Axis::default()
						.title(Span::styled(cfg.name(app::Axis::Y), Style::default().fg(Color::Cyan)))
						.style(Style::default().fg(cfg.axis_color))
						.bounds(cfg.bounds(app::Axis::Y)));
				f.render_widget(chart, size)
			})?;
		}

		if let Some(Event::Key(key)) = poll_event()? {
			match key.modifiers {
				KeyModifiers::CONTROL => {
					match key.code {
						KeyCode::Char('c') => break,
						KeyCode::Char('+') => cfg.update_scale(100),
						KeyCode::Char('-') => cfg.update_scale(-100),
						_ => {},
					}
				},
				_ => {
					match key.code {
						KeyCode::Char('q') => break,
						KeyCode::Char(' ') => pause = !pause,
						KeyCode::Char('+') => cfg.update_scale(1000),
						KeyCode::Char('-') => cfg.update_scale(-1000),
						KeyCode::Char('v') => cfg.set_vectorscope(!cfg.vectorscope()),
						KeyCode::Char('s') => cfg.set_scatter(!cfg.scatter()),
						_ => {},
					}
				}
			}
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
