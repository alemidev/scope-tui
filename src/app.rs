
use std::{io::{self, ErrorKind}, time::{Duration, Instant}};
use tui::{
	style::Color, widgets::{GraphType, Table, Row, Cell}, symbols,
	backend::Backend,
	widgets::{Chart, Axis, Dataset},
	Terminal, text::Span, style::{Style, Modifier}, layout::{Rect, Constraint}
};
use crossterm::event::{self, Event, KeyCode, KeyModifiers};

use libpulse_simple_binding::Simple;
use libpulse_binding::{stream::Direction, def::BufferAttr};
use libpulse_binding::sample::{Spec, Format};


use crate::Args;
use crate::config::{ChartNames, ChartBounds, ChartReferences, AppConfig, Dimension};
use crate::parser::{SampleParser, Signed16PCM};

pub struct App {
	pub cfg: AppConfig,
	pub references: ChartReferences,
	pub bounds: ChartBounds,
	pub names: ChartNames,
}

impl App {
	pub fn update_values(&mut self) {
		if self.cfg.scale > 32770 { // sample max value is 32768 (32 bits), but we leave 2 pixels for
			self.cfg.scale = 32770; //  padding (and to not "disaling" range when reaching limit)
		}
		if self.cfg.scale < 0 {
			self.cfg.scale = 0;
		}
		if self.cfg.vectorscope {
			self.names.x = "left -".into();
			self.names.y = "| right".into();
			self.bounds.x = [-(self.cfg.scale as f64), self.cfg.scale as f64];
			self.bounds.y = [-(self.cfg.scale as f64), self.cfg.scale as f64];
			self.references.x = vec![(-(self.cfg.scale as f64), 0.0), (self.cfg.scale as f64, 0.0)];
			self.references.y = vec![(0.0, -(self.cfg.scale as f64)), (0.0, self.cfg.scale as f64)];
		} else {
			self.names.x = "time -".into();
			self.names.y = "| amplitude".into();
			self.bounds.x = [0.0, self.cfg.width as f64];
			self.bounds.y = [-(self.cfg.scale as f64), self.cfg.scale as f64];
			self.references.x = vec![(0.0, 0.0), (self.cfg.width as f64, 0.0)];
			let half_width = self.cfg.width as f64 / 2.0;
			self.references.y = vec![(half_width, -(self.cfg.scale as f64)), (half_width, self.cfg.scale as f64)];
		}
	}

	pub fn marker_type(&self) -> symbols::Marker {
		if self.cfg.braille {
			symbols::Marker::Braille
		} else {
			symbols::Marker::Dot
		}
	}

	pub fn graph_type(&self) -> GraphType {
		if self.cfg.scatter {
			GraphType::Scatter
		} else {
			GraphType::Line
		}
	}

	pub fn palette(&self, index: usize) -> Color {
		*self.cfg.palette.get(index % self.cfg.palette.len()).unwrap_or(&Color::White)
	}
}

impl From::<&crate::Args> for App {
	fn from(args: &crate::Args) -> Self {
		let cfg = AppConfig {
			axis_color: Color::DarkGray,
			palette: vec![Color::Red, Color::Yellow, Color::Green, Color::Magenta],
			scale: args.range,
			width: args.buffer / (2 * args.channels as u32), // TODO also make bit depth customizable
			triggering: args.triggering,
			threshold: args.threshold,
			vectorscope: args.vectorscope,
			references: !args.no_reference,
			show_ui: !args.no_ui,
			braille: !args.no_braille,
			scatter: args.scatter,
		};

		let mut app = App {
			cfg,
			references: ChartReferences::default(),
			bounds: ChartBounds::default(),
			names: ChartNames::default(),
		};

		app.update_values();

		app
	}
}

pub fn run_app<T : Backend>(args: Args, terminal: &mut Terminal<T>) -> Result<(), io::Error> {
	// prepare globals
	let mut buffer : Vec<u8> = vec![0; args.buffer as usize];
	let mut app = App::from(&args);
	let fmt = Signed16PCM{}; // TODO some way to choose this?

	let mut pause = false;

	// setup audio capture
	let spec = Spec {
		format: Format::S16NE,
		channels: args.channels,
		rate: args.sample_rate,
	};
	assert!(spec.is_valid());

	let dev = match &args.device {
		Some(d) => Some(d.as_str()),
		None => None,
	};

	let s = match Simple::new(
		None,                // Use the default server
		"scope-tui",         // Our application’s name
		Direction::Record,   // We want a record stream
		dev,                 // Use requested device, or default
		"data",              // Description of our stream
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
			channels = fmt.oscilloscope(&mut buffer, args.channels);

			if app.cfg.triggering {
				// TODO allow to customize channel to use for triggering
				if let Some(ch) = channels.get(0) {
					let mut discard = 0;
					for i in 0..ch.len()-1 { // seek to first sample rising through threshold
						if ch[i] <= app.cfg.threshold && ch[i+1] > app.cfg.threshold { // triggered
							break;
						} else {
							discard += 1;
						}
					}
					for ch in channels.iter_mut() {
						let limit = if ch.len() < discard { ch.len() } else { discard };
						*ch = ch[limit..].to_vec();
					}
				}
			}
		}

		let samples = channels.iter().map(|x| x.len()).max().unwrap_or(0);

		let mut measures : Vec<(String, Vec<(f64, f64)>)>;

		// This third buffer is kinda weird because of lifetimes on Datasets, TODO
		//  would be nice to make it more straight forward instead of this deep tuple magic
		if app.cfg.vectorscope {
			measures = vec![];
			for (i, chunk) in channels.chunks(2).enumerate() {
				let mut tmp = vec![];
				match chunk.len() {
					2 => {
						for i in 0..std::cmp::min(chunk[0].len(), chunk[0].len()) {
							tmp.push((chunk[0][i] as f64, chunk[1][i] as f64));
						}
					},
					1 => {
						for i in 0..chunk[0].len() {
							tmp.push((chunk[0][i] as f64, i as f64));
						}
					},
					_ => continue,
				}
				// split it in two so the math downwards still works the same
				let pivot = tmp.len() / 2;
				measures.push((channel_name(i * 2, true), tmp[pivot..].to_vec())); // put more recent first
				measures.push((channel_name((i * 2) + 1, true), tmp[..pivot].to_vec()));
			}
		} else {
			measures = vec![];
			for (i, channel) in channels.iter().enumerate() {
				let mut tmp = vec![];
				for i in 0..channel.len() {
					tmp.push((i as f64, channel[i]));
				}
				measures.push((channel_name(i, false), tmp));
			}
		}

		let mut datasets = vec![];

		if app.cfg.references {
			datasets.push(data_set("", &app.references.x, app.marker_type(), GraphType::Line, app.cfg.axis_color));
			datasets.push(data_set("", &app.references.y, app.marker_type(), GraphType::Line, app.cfg.axis_color));
		}

		let trigger_pt = [(0.0, app.cfg.threshold)];
		datasets.push(data_set("T", &trigger_pt, app.marker_type(), GraphType::Scatter, Color::Cyan));

		let m_len = measures.len() - 1;
		for (i, (name, ds)) in measures.iter().rev().enumerate() {
			datasets.push(data_set(&name, ds, app.marker_type(), app.graph_type(), app.palette(m_len - i)));
		}

		fps += 1;

		if last_poll.elapsed().as_secs() >= 1 {
			framerate = fps;
			fps = 0;
			last_poll = Instant::now();
		}

		terminal.draw(|f| {
			let mut size = f.size();
			if app.cfg.show_ui {
				let heading = header(&app, samples as u32, framerate);
				f.render_widget(heading, Rect { x: size.x, y: size.y, width: size.width, height:1 });
				size.height -= 1;
				size.y += 1;
			}
			let chart = Chart::new(datasets)
				.x_axis(axis(&app, Dimension::X)) // TODO allow to have axis sometimes?
				.y_axis(axis(&app, Dimension::Y));
			f.render_widget(chart, size)
		})?;

		if let Some(Event::Key(key)) = poll_event()? {
			match key.modifiers {
				KeyModifiers::SHIFT => {
					match key.code {
						KeyCode::Up       => app.cfg.scale     -= 1000, // inverted to act as zoom
						KeyCode::Down     => app.cfg.scale     += 1000, // inverted to act as zoom
						KeyCode::Right    => app.cfg.width     += 100,
						KeyCode::Left     => app.cfg.width     -= 100,
						KeyCode::PageUp   => app.cfg.threshold += 1000.0,
						KeyCode::PageDown => app.cfg.threshold -= 1000.0,
						_ => {},
					}
				},
				KeyModifiers::CONTROL => {
					match key.code { // mimic other programs shortcuts to quit, for user friendlyness
						KeyCode::Char('c') | KeyCode::Char('q') | KeyCode::Char('w') => break,
						KeyCode::Up       => app.cfg.scale     -= 10, // inverted to act as zoom
						KeyCode::Down     => app.cfg.scale     += 10, // inverted to act as zoom
						KeyCode::Right    => app.cfg.width     += 1,
						KeyCode::Left     => app.cfg.width     -= 1,
						KeyCode::PageUp   => app.cfg.threshold += 10.0,
						KeyCode::PageDown => app.cfg.threshold -= 10.0,
						KeyCode::Char('r') => { // reset settings
							app.cfg.references  = !args.no_reference;
							app.cfg.show_ui     = !args.no_ui;
							app.cfg.braille     = !args.no_braille;
							app.cfg.threshold   = args.threshold;
							app.cfg.width       = args.buffer / (args.channels as u32 * 2); // TODO ...
							app.cfg.scale       = args.range;
							app.cfg.vectorscope = args.vectorscope;
							app.cfg.triggering  = args.triggering;
						},
						_ => {},
					}
				},
				_ => {
					match key.code {
						KeyCode::Char('q') => break,
						KeyCode::Char(' ') => pause = !pause,
						KeyCode::Char('v') => app.cfg.vectorscope = !app.cfg.vectorscope,
						KeyCode::Char('s') => app.cfg.scatter     = !app.cfg.scatter,
						KeyCode::Char('b') => app.cfg.braille     = !app.cfg.braille,
						KeyCode::Char('h') => app.cfg.show_ui     = !app.cfg.show_ui,
						KeyCode::Char('r') => app.cfg.references  = !app.cfg.references,
						KeyCode::Char('t') => app.cfg.triggering  = !app.cfg.triggering,
						KeyCode::Up       => app.cfg.scale     -= 250, // inverted to act as zoom
						KeyCode::Down     => app.cfg.scale     += 250, // inverted to act as zoom
						KeyCode::Right    => app.cfg.width     += 25,
						KeyCode::Left     => app.cfg.width     -= 25,
						KeyCode::PageUp   => app.cfg.threshold += 250.0,
						KeyCode::PageDown => app.cfg.threshold -= 250.0,
						KeyCode::Tab => { // only reset "zoom"
							app.cfg.width       = args.buffer / (args.channels as u32 * 2); // TODO ...
							app.cfg.scale       = args.range;
						},
						KeyCode::Esc      => { // back to oscilloscope
							app.cfg.references  = !args.no_reference;
							app.cfg.show_ui     = !args.no_ui;
							app.cfg.vectorscope = args.vectorscope;
						},
						_ => {},
					}
				}
			}
			app.update_values();
		}
	}

	Ok(())
}

// TODO these functions probably shouldn't be here

fn header(app: &App, samples: u32, framerate: u32) -> Table<'static> {
	Table::new(
		vec![
			Row::new(
				vec![
					Cell::from(format!("TUI {}", if app.cfg.vectorscope { "Vectorscope" } else { "Oscilloscope" })).style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
					Cell::from(format!("{} plot", if app.cfg.scatter { "scatter" } else { "line" })),
					Cell::from(format!("{}", if app.cfg.triggering { "triggered" } else { "live" } )),
					Cell::from(format!("threshold {:.0} ^", app.cfg.threshold)),
					Cell::from(format!("range +{}-", app.cfg.scale)),
					Cell::from(format!("{}/{} samples", samples as u32, app.cfg.width)),
					Cell::from(format!("{}fps", framerate)),
				]
			)
		]
	)
	.style(Style::default().fg(Color::Cyan))
	.widths(&[
		Constraint::Percentage(32),
		Constraint::Percentage(12),
		Constraint::Percentage(12),
		Constraint::Percentage(12),
		Constraint::Percentage(12),
		Constraint::Percentage(12),
		Constraint::Percentage(6)
	])
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

fn axis(app: &App, dim: Dimension) -> Axis {
	let (name, bounds) = match dim {
		Dimension::X => (&app.names.x, &app.bounds.x),
		Dimension::Y => (&app.names.y, &app.bounds.y),
	};
	let mut a = Axis::default();
	if app.cfg.show_ui {
		a = a.title(Span::styled(name, Style::default().fg(Color::Cyan)));
	}
	a.style(Style::default().fg(app.cfg.axis_color)).bounds(*bounds)
}

fn channel_name(index: usize, vectorscope: bool) -> String {
	if vectorscope { return format!("{}", index); }
	match index {
		0 => "L".into(),
		1 => "R".into(),
		_ => format!("{}", index),
	}
}
