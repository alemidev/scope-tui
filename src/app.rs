
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

	pub fn bounds(&self, axis: &Dimension) -> [f64;2] {
		match axis {
			Dimension::X => self.bounds.x,
			Dimension::Y => self.bounds.y,
		}
	}

	pub fn name(&self, axis: &Dimension) -> &str {
		match axis {
			Dimension::X => self.names.x.as_str(),
			Dimension::Y => self.names.y.as_str(),
		}
	}

	pub fn scatter(&self) -> bool {
		match self.cfg.graph_type {
			GraphType::Scatter => true,
			_ => false,
		}
	}

	// pub fn references(&self) -> Vec<Dataset> {
	// 	vec![
	// 		Dataset::default()
	// 			.name("")
	// 			.marker(self.cfg.marker_type)
	// 			.graph_type(GraphType::Line)
	// 			.style(Style::default().fg(self.cfg.axis_color))
	// 			.data(&self.references.x),
	// 		Dataset::default()
	// 			.name("")
	// 			.marker(self.cfg.marker_type)
	// 			.graph_type(GraphType::Line)
	// 			.style(Style::default().fg(self.cfg.axis_color))
	// 			.data(&self.references.y),
	// 	]
	// }

	pub fn update_scale(&mut self, increment: i32) {
		if increment > 0 || increment.abs() < self.cfg.scale as i32 {
			self.cfg.scale = ((self.cfg.scale as i32) + increment) as u32;
			self.update_values();
		}
	}

	pub fn set_scatter(&mut self, scatter: bool) {
		self.cfg.graph_type = if scatter { GraphType::Scatter } else { GraphType::Line };
	}
}

impl From::<&crate::Args> for App {
	fn from(args: &crate::Args) -> Self {
		let marker_type = if args.no_braille { symbols::Marker::Dot } else { symbols::Marker::Braille };
		let graph_type  = if args.scatter    { GraphType::Scatter   } else { GraphType::Line          };

		let cfg = AppConfig {
			title: "TUI Oscilloscope  --  <me@alemi.dev>".into(),
			axis_color: Color::DarkGray,
			palette: vec![Color::Red, Color::Yellow],
			scale: args.range,
			width: args.buffer / 4, // TODO It's 4 because 2 channels and 2 bytes per sample!
			triggering: args.triggering,
			threshold: args.threshold,
			vectorscope: args.vectorscope,
			references: !args.no_reference,
			marker_type, graph_type,
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

		if app.cfg.triggering {
			// TODO allow to customize channel to use for triggering and threshold
			if let Some(ch) = channels.get(0) {
				let mut discard = 0;
				for i in 0..ch.len() { // seek to first sample rising through threshold
					if i + 1 < ch.len() && ch[i] <= app.cfg.threshold && ch[i+1] > app.cfg.threshold { // triggered
						break;
					} else {
						discard += 1;
					}
				}
				for ch in channels.iter_mut() {
					*ch = ch[discard..].to_vec();
				}
			}
		}

		let samples = channels.iter().map(|x| x.len()).max().unwrap_or(0);

		let mut measures;

		if app.cfg.vectorscope {
			measures = vec![];
			for chunk in channels.chunks(2) {
				let mut tmp = vec![];
				for i in 0..chunk[0].len() {
					tmp.push((chunk[0][i] as f64, chunk[1][i] as f64));
				}
				// split it in two so the math downwards still works the same
				let pivot = tmp.len() / 2;
				measures.push(tmp[pivot..].to_vec()); // put more recent first
				measures.push(tmp[..pivot].to_vec());
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
		let trigger_pt;

		if app.cfg.references {
			trigger_pt = [(0.0, app.cfg.threshold)];
			datasets.push(data_set("", &app.references.x, app.cfg.marker_type, GraphType::Line, app.cfg.axis_color));
			datasets.push(data_set("", &app.references.y, app.cfg.marker_type, GraphType::Line, app.cfg.axis_color));
			datasets.push(data_set("T", &trigger_pt, app.cfg.marker_type, GraphType::Scatter, Color::Cyan));
		}

		let ds_names = if app.cfg.vectorscope { vec!["1", "2"] } else { vec!["R", "L"] };
		let palette : Vec<Color> = app.cfg.palette.iter().rev().map(|x| x.clone()).collect();

		for (i, ds) in measures.iter().rev().enumerate() {
			datasets.push(data_set(ds_names[i], ds, app.cfg.marker_type, app.cfg.graph_type, palette[i % palette.len()]));
		}

		fps += 1;

		if last_poll.elapsed().as_secs() >= 1 {
			framerate = fps;
			fps = 0;
			last_poll = Instant::now();
		}

		terminal.draw(|f| {
			let mut size = f.size();
			if app.cfg.references {
				let heading = Table::new(
					vec![
						Row::new(
							vec![
								Cell::from(format!("TUI {}", if app.cfg.vectorscope { "Vectorscope" } else { "Oscilloscope" })).style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
								Cell::from(format!("{}{} mode", if app.cfg.triggering { "triggered " } else { "" }, if app.scatter() { "scatter" } else { "line" })),
								Cell::from(format!("range +-{}", app.cfg.scale)),
								Cell::from(format!("{}smpl", samples as u32)),
								Cell::from(format!("{:.1}kHz", args.sample_rate as f32 / 1000.0)),
								Cell::from(format!("{}fps", framerate)),
							]
						)
					]
				)
				.style(Style::default().fg(Color::Cyan))
				.widths(&[Constraint::Percentage(40), Constraint::Percentage(20), Constraint::Percentage(20), Constraint::Percentage(7), Constraint::Percentage(7), Constraint::Percentage(6)]);
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
						KeyCode::Char('v') => app.cfg.vectorscope = !app.cfg.vectorscope,
						KeyCode::Char('s') => app.set_scatter(!app.scatter()), // TODO no funcs
						KeyCode::Char('h') => app.cfg.references = !app.cfg.references,
						KeyCode::Char('t') => app.cfg.triggering = !app.cfg.triggering,
						KeyCode::Up        => app.cfg.threshold += 100.0,
						KeyCode::Down      => app.cfg.threshold -= 100.0,
						KeyCode::PageUp    => app.cfg.threshold += 1000.0,
						KeyCode::PageDown  => app.cfg.threshold -= 1000.0,
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
	let mut a = Axis::default();
	if app.cfg.references {
		a = a.title(Span::styled(app.name(&dim), Style::default().fg(Color::Cyan)));
	}
	a.style(Style::default().fg(app.cfg.axis_color))
		.bounds(app.bounds(&dim))
}
