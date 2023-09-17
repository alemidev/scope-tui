
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

use crate::{Args, source::{PulseAudioSimple, DataSource}};
use crate::config::{ChartNames, ChartBounds, ChartReferences, AppConfig, Dimension};
use crate::parser::{SampleParser, Signed16PCM};

pub fn run_app<T : Backend>(args: Args, terminal: &mut Terminal<T>) -> Result<(), io::Error> {
	// prepare globals
	let mut app = App::from(&args);
	let fmt = Signed16PCM{}; // TODO some way to choose this?
	let mut source = PulseAudioSimple::new(
		args.device.as_deref(),
		args.channels,
		args.sample_rate,
		args.buffer,
		args.server_buffer
	).unwrap();

	let mut fps = 0;
	let mut framerate = 0;
	let mut last_poll = Instant::now();
	let mut channels = vec![];

	loop {
		let data = source.recv().unwrap();

		if !app.cfg.pause {
			channels = fmt.oscilloscope(data, args.channels);
		}

		let mut trigger_offset = 0;

		if app.cfg.triggering {
			// TODO allow to customize channel to use for triggering
			if let Some(ch) = channels.get(0) {
				for i in 0..ch.len() { // seek to first sample rising through threshold
					if triggered(ch, i, app.cfg.threshold, app.cfg.depth, app.cfg.falling_edge) { // triggered
						break;
					} else {
						trigger_offset += 1;
					}
				}
				// for ch in channels.iter_mut() {
				// 	let limit = if ch.len() < discard { ch.len() } else { discard };
				// 	*ch = ch[limit..].to_vec();
				// }
			}
		}

		let mut measures : Vec<(String, Vec<(f64, f64)>)> = vec![];
		let mut peaks : Vec<Vec<(f64, f64)>> = vec![];

		// This third buffer is kinda weird because of lifetimes on Datasets, TODO
		//  would be nice to make it more straight forward instead of this deep tuple magic
		if app.cfg.vectorscope {
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
			for (i, channel) in channels.iter().enumerate() {
				let mut tmp = vec![];
				let mut peak_up = 0.0;
				let mut peak_down = 0.0;
				for i in 0..channel.len() {
					if i >= trigger_offset {
						tmp.push(((i - trigger_offset) as f64, channel[i]));
					}
					if channel[i] > peak_up {
						peak_up = channel[i];
					}
					if channel[i] < peak_down {
						peak_down = channel[i];
					}
				}
				measures.push((channel_name(i, false), tmp));
				peaks.push(vec![(0.0, peak_down), (0.0, peak_up)]);
			}
		}

		let samples = measures.iter().map(|(_,x)| x.len()).max().unwrap_or(0);

		let mut datasets = vec![];

		if app.cfg.references {
			datasets.push(data_set("", &app.references.x, app.marker_type(), GraphType::Line, app.cfg.axis_color));
			datasets.push(data_set("", &app.references.y, app.marker_type(), GraphType::Line, app.cfg.axis_color));
		}

		let trigger_pt;
		if app.cfg.triggering {
			trigger_pt = [(0.0, app.cfg.threshold)];
			datasets.push(data_set("T", &trigger_pt, app.marker_type(), GraphType::Scatter, Color::Cyan));
		}

		let m_len = measures.len() - 1;

		if !app.cfg.vectorscope && app.cfg.peaks {
			for (i, pt) in peaks.iter().rev().enumerate() {
				datasets.push(data_set("", pt, app.marker_type(), GraphType::Scatter, app.palette(m_len - i)));
			}
		}

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

		if process_events(&mut app, &args)? {
			break;
		}
	}

	Ok(())
}

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
		if self.cfg.depth < 1 {
			self.cfg.depth = 1;
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
			depth: args.check_depth,
			threshold: args.threshold,
			vectorscope: args.vectorscope,
			references: !args.no_reference,
			show_ui: !args.no_ui,
			braille: !args.no_braille,
			scatter: args.scatter,
			falling_edge: args.falling_edge,
			peaks: args.show_peaks,
			pause: false,
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
// TODO these functions probably shouldn't be here

fn header(app: &App, samples: u32, framerate: u32) -> Table<'static> {
	Table::new(
		vec![
			Row::new(
				vec![
					Cell::from(format!("TUI {}", if app.cfg.vectorscope { "Vectorscope" } else { "Oscilloscope" })).style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
					Cell::from(format!("{}",
						if app.cfg.triggering {
							format!(
								"{} {:.0}{} trigger",
								if app.cfg.falling_edge { "v" } else { "^" },
								app.cfg.threshold,
								if app.cfg.depth > 1 { format!(":{}", app.cfg.depth) } else { "".into() },
							)
						} else {
							"live".into()
						}
					)),
					Cell::from(format!("-{}+ range", app.cfg.scale)),
					Cell::from(format!("{}/{} sample", app.cfg.width, samples as u32)),
					Cell::from(format!("{}fps", framerate)),
					Cell::from(format!("{}{}", if app.cfg.peaks { "|" } else { " " }, if app.cfg.scatter { "***" } else { "---" })),
					Cell::from(format!("{}", if app.cfg.pause { "||" } else { "|>" } )),
				]
			)
		]
	)
	.style(Style::default().fg(Color::Cyan))
	.widths(&[
		Constraint::Percentage(35),
		Constraint::Percentage(15),
		Constraint::Percentage(15),
		Constraint::Percentage(15),
		Constraint::Percentage(6),
		Constraint::Percentage(6),
		Constraint::Percentage(6)
	])
}

fn process_events(app: &mut App, args: &Args) -> Result<bool, io::Error> {
	let mut quit = false;

	if event::poll(Duration::from_millis(0))? { // process all enqueued events
		let event = event::read()?;

		match event {
			Event::Key(key) => {
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
							KeyCode::Char('c') | KeyCode::Char('q') | KeyCode::Char('w') => quit = true,
							KeyCode::Up       => app.cfg.scale     -= 50, // inverted to act as zoom
							KeyCode::Down     => app.cfg.scale     += 50, // inverted to act as zoom
							KeyCode::Right    => app.cfg.width     += 5,
							KeyCode::Left     => app.cfg.width     -= 5,
							KeyCode::PageUp   => app.cfg.threshold += 50.0,
							KeyCode::PageDown => app.cfg.threshold -= 50.0,
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
							KeyCode::Char('q') => quit = true,
							KeyCode::Char(' ') => app.cfg.pause        = !app.cfg.pause,
							KeyCode::Char('v') => app.cfg.vectorscope  = !app.cfg.vectorscope,
							KeyCode::Char('s') => app.cfg.scatter      = !app.cfg.scatter,
							KeyCode::Char('b') => app.cfg.braille      = !app.cfg.braille,
							KeyCode::Char('h') => app.cfg.show_ui      = !app.cfg.show_ui,
							KeyCode::Char('r') => app.cfg.references   = !app.cfg.references,
							KeyCode::Char('e') => app.cfg.falling_edge = !app.cfg.falling_edge,
							KeyCode::Char('t') => app.cfg.triggering   = !app.cfg.triggering,
							KeyCode::Char('p') => app.cfg.peaks        = !app.cfg.peaks,
							KeyCode::Char('=') => app.cfg.depth       += 1,
							KeyCode::Char('-') => app.cfg.depth       -= 1,
							KeyCode::Char('+') => app.cfg.depth       += 10,
							KeyCode::Char('_') => app.cfg.depth       -= 10,
							KeyCode::Up       => app.cfg.scale        -= 250, // inverted to act as zoom
							KeyCode::Down     => app.cfg.scale        += 250, // inverted to act as zoom
							KeyCode::Right    => app.cfg.width        += 25,
							KeyCode::Left     => app.cfg.width        -= 25,
							KeyCode::PageUp   => app.cfg.threshold    += 250.0,
							KeyCode::PageDown => app.cfg.threshold    -= 250.0,
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
			},
			_ => {},
		};
	}

	Ok(quit)
}

// TODO can this be made nicer?
fn triggered(data: &[f64], index: usize, threshold: f64, depth: u32, falling_edge:bool) -> bool {
	if data.len() < index + (1+depth as usize) { return false; }
	if falling_edge {
		if data[index] >= threshold {
			for i in 1..=depth as usize {
				if data[index+i] >= threshold {
					return false;
				}
			}
			return true;
		} else {
			return false;
		}
	} else {
		if data[index] <= threshold {
			for i in 1..=depth as usize {
				if data[index+i] <= threshold {
					return false;
				}
			}
			return true;
		} else {
			return false;
		}
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
