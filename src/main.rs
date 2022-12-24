use std::{io, time::Duration};
use tui::{
	backend::CrosstermBackend,
	widgets::{Block, Chart, Axis, GraphType, Dataset, BorderType},
	// layout::{Layout, Constraint, Direction},
	Terminal, text::Span, style::{Style, Color}, symbols
};
use crossterm::{
	event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
	execute,
	terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use libpulse_simple_binding::Simple;
use libpulse_binding::{stream::Direction, def::BufferAttr};
use libpulse_binding::sample::{Spec, Format};

use clap::Parser;

/// A simple oscilloscope/vectorscope for your terminal
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
	/// Size of audio buffer, and width of scope
	width: u32,
	
	/// Audio device to attach to
	#[arg(short, long)]
	device: Option<String>,

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

trait SampleParser {
	fn oscilloscope(data: &mut [u8]) -> (Vec<(f64, f64)>, Vec<(f64, f64)>);
	fn vectorscope (data: &mut [u8]) -> Vec<(f64, f64)>;
}

struct Signed16PCM {}

impl SampleParser for Signed16PCM {
	fn oscilloscope(data: &mut [u8]) -> (Vec<(f64, f64)>, Vec<(f64, f64)>) { 
		let mut left = Vec::new(); // TODO does left really come first?
		let mut right = Vec::new();
		let mut buf : i16 = 0;
		let mut count : f64 = 0.0;
		let mut flip = false;
		let mut side = false;
		for sample in data {
			if flip {
				buf |= (*sample as i16) << 8;
				if side {
					left.push((count, buf as f64));
				} else {
					right.push((count, buf as f64));
					count += 1.0;
				}
				buf = 0;
				side = !side;
			} else {
				buf |= *sample as i16;
			}
			flip = !flip;
		}
		(left, right)
	}

	fn vectorscope(data: &mut [u8]) -> Vec<(f64, f64)> { 
		let mut out = Vec::new(); // TODO does left really come first?
		let mut buf : i16 = 0;
		let mut flip = false;
		let mut point = None;
		for sample in data {
			if flip {
				buf |= (*sample as i16) << 8;
				if point.is_none() {
					point = Some(buf as f64);
				} else {
					out.push((point.unwrap(), buf as f64));
					point = None;
				}
				buf = 0;
			} else {
				buf |= *sample as i16;
			}
			flip = !flip;
		}
		out
	}
}

fn poll_event() -> Result<Option<Event>, std::io::Error> {
	if event::poll(Duration::from_millis(0))? {
		Ok(Some(event::read()?))
	} else {
		Ok(None)
	}
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

	let marker_type = if args.no_braille { symbols::Marker::Dot } else { symbols::Marker::Braille };
	let graph_type  = if args.scatter    { GraphType::Scatter   } else { GraphType::Line          };

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
	execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
	let backend = CrosstermBackend::new(stdout);
	let mut terminal = Terminal::new(backend)?;
	terminal.hide_cursor().unwrap();

	let mut buffer : Vec<u8> = vec![0; args.width as usize];
	// let mut buffer : [u8; WINDOW] = [0;WINDOW];
	let y_bounds : [f64; 2];
	let x_bounds : [f64; 2];
	let reference_x : Dataset;
	let reference_y : Dataset;

	let mut ref_data_x = Vec::new();
	let mut ref_data_y = Vec::new();

	let mut pause = false;

	if args.vectorscope {
		x_bounds = [-(args.scale as f64), args.scale as f64];
		y_bounds = [-(args.scale as f64), args.scale as f64];

		for x in -(args.scale as i64)..(args.scale as i64) {
			ref_data_x.push((x as f64, 0 as f64));
			ref_data_y.push((0 as f64, x as f64));
		}
	} else {
		x_bounds = [0.0, args.width as f64 / 4.0];
		y_bounds = [-(args.scale as f64), args.scale as f64];

		for x in 0..args.width/4 {
			ref_data_x.push((x as f64, 0 as f64));
		}
		for y in -(args.scale as i64)..(args.scale as i64) {
			ref_data_y.push(((args.width as f64) / 8.0, y as f64));
		}
	}

	reference_x = Dataset::default()
			.name("X")
			.marker(marker_type)
			.graph_type(GraphType::Line)
			.style(Style::default().fg(Color::DarkGray))
			.data(&ref_data_x);
	reference_y = Dataset::default()
			.name("Y")
			.marker(marker_type)
			.graph_type(GraphType::Line)
			.style(Style::default().fg(Color::DarkGray))
			.data(&ref_data_y);

	loop {
		s.read(&mut buffer).unwrap();

		let mut datasets = vec![];
		let (left, right) : (Vec<(f64, f64)>, Vec<(f64, f64)>);
		let merged : Vec<(f64, f64)>;
		let labels_x : Vec<Span>;
		let labels_y : Vec<Span>;
		let title_x : String;
		let title_y : String;

		if !args.no_reference {
			datasets.push(reference_x.clone());
			datasets.push(reference_y.clone());
		}

		if args.vectorscope {
			merged = Signed16PCM::vectorscope(&mut buffer);
			datasets.push(
				Dataset::default()
					.name("V")
					.marker(marker_type)
					.graph_type(graph_type)
					.style(Style::default().fg(Color::Red))
					.data(&merged)
			);
			labels_x = vec![Span::from("-"), Span::from("0"), Span::from("+")];
			labels_y = vec![Span::from("-"), Span::from("0"), Span::from("+")];
			title_x = "left".into();
			title_y = "right".into();
		} else {
			(left, right) = Signed16PCM::oscilloscope(&mut buffer);
			datasets.push(
				Dataset::default()
					.name("R")
					.marker(marker_type)
					.graph_type(graph_type)
					.style(Style::default().fg(Color::Yellow))
					.data(&right)
			);
			datasets.push(
				Dataset::default()
					.name("L")
					.marker(marker_type)
					.graph_type(graph_type)
					.style(Style::default().fg(Color::Red))
					.data(&left)
			);
			labels_x = vec![Span::from("0"), Span::from(format!("{}", args.width / 4))];
			labels_y = vec![Span::from("-"), Span::from("0"), Span::from("+")];
			title_x = "sample".into();
			title_y = "amplitude".into();
		}

		if !pause {
			terminal.draw(|f| {
				let size = f.size();
				let chart = Chart::new(datasets)
					.block(Block::default()
						.border_type(BorderType::Rounded)
						.border_style(Style::default().fg(Color::DarkGray))
						.title(Span::styled("TUI Oscilloscope  --  <me@alemi.dev>", Style::default().fg(Color::Cyan))))
					.x_axis(Axis::default()
						.title(Span::styled(title_x.as_str(), Style::default().fg(Color::Cyan)))
						.style(Style::default().fg(Color::DarkGray))
						.bounds(x_bounds)
						.labels(labels_x))
					.y_axis(Axis::default()
						.title(Span::styled(title_y.as_str(), Style::default().fg(Color::Cyan)))
						.style(Style::default().fg(Color::DarkGray))
						.bounds(y_bounds)
						.labels(labels_y));
				f.render_widget(chart, size)
			})?;
		}

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
