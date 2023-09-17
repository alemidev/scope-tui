
use std::{io, time::{Duration, Instant}, ops::Range};
use ratatui::{
	style::Color, widgets::{Table, Row, Cell}, symbols::Marker,
	backend::Backend,
	widgets::Chart,
	Terminal, style::{Style, Modifier}, layout::{Rect, Constraint}
};
use crossterm::event::{self, Event, KeyCode, KeyModifiers};

use crate::{source::DataSource, display::{GraphConfig, oscilloscope::Oscilloscope, DisplayMode, Dimension, vectorscope::Vectorscope, spectroscope::Spectroscope}};
use crate::parser::{SampleParser, Signed16PCM};

pub enum CurrentDisplayMode {
	Oscilloscope,
	Vectorscope,
	Spectroscope,
}

pub struct App {
	pause: bool,
	channels: u8,
	graph: GraphConfig,
	oscilloscope: Oscilloscope,
	vectorscope: Vectorscope,
	spectroscope: Spectroscope,
	mode: CurrentDisplayMode,
}

impl App {
	pub fn run<T : Backend>(&mut self, mut source: impl DataSource, terminal: &mut Terminal<T>) -> Result<(), io::Error> {
		// prepare globals
		let fmt = Signed16PCM{}; // TODO some way to choose this?
	
		let mut fps = 0;
		let mut framerate = 0;
		let mut last_poll = Instant::now();
		let mut channels = vec![];
	
		loop {
			let data = source.recv().unwrap();
	
			if !self.pause {
				channels = fmt.oscilloscope(data, self.channels);
			}
	
			fps += 1;
	
			if last_poll.elapsed().as_secs() >= 1 {
				framerate = fps;
				fps = 0;
				last_poll = Instant::now();
			}
	
			{
				let display = match self.mode {
					CurrentDisplayMode::Oscilloscope => &self.oscilloscope as &dyn DisplayMode,
					CurrentDisplayMode::Vectorscope => &self.vectorscope as &dyn DisplayMode,
					CurrentDisplayMode::Spectroscope => &self.spectroscope as &dyn DisplayMode,
				};
	
				let mut datasets = Vec::new();
				if self.graph.references {
					datasets.append(&mut display.references(&self.graph));
				}
				datasets.append(&mut display.process(&self.graph, &channels));
				terminal.draw(|f| {
					let mut size = f.size();
					if self.graph.show_ui {
						f.render_widget(
							make_header(&self.graph, &display.header(&self.graph), framerate, self.pause),
							Rect { x: size.x, y: size.y, width: size.width, height:1 } // a 1px line at the top
						);
						size.height -= 1;
						size.y += 1;
					}
					let chart = Chart::new(datasets.iter().map(|x| x.into()).collect())
						.x_axis(display.axis(&self.graph, Dimension::X)) // TODO allow to have axis sometimes?
						.y_axis(display.axis(&self.graph, Dimension::Y));
					f.render_widget(chart, size)
				}).unwrap();
			}

			while event::poll(Duration::from_millis(0))? { // process all enqueued events
				let event = event::read()?;

				if self.process_events(event.clone())? { return Ok(()); }
				self.current_display().handle(event);
			}
		}
	}

	fn current_display(&mut self) -> &mut dyn DisplayMode {
		match self.mode {
			CurrentDisplayMode::Oscilloscope => &mut self.oscilloscope as &mut dyn DisplayMode,
			CurrentDisplayMode::Vectorscope => &mut self.vectorscope as &mut dyn DisplayMode,
			CurrentDisplayMode::Spectroscope => &mut self.spectroscope as &mut dyn DisplayMode,
		}
	}

	fn process_events(&mut self, event: Event) -> Result<bool, io::Error> {
		let mut quit = false;
		if let Event::Key(key) = event {
			if let KeyModifiers::CONTROL = key.modifiers {
				match key.code { // mimic other programs shortcuts to quit, for user friendlyness
					KeyCode::Char('c') | KeyCode::Char('q') | KeyCode::Char('w') => quit = true,
					_ => {},
				}
			}
			let magnitude = match key.modifiers {
				KeyModifiers::SHIFT => 10.0,
				KeyModifiers::CONTROL => 5.0,
				KeyModifiers::ALT => 0.2,
				_ => 1.0,
			};
			match key.code {
				KeyCode::Up       => update_value_i(&mut self.graph.scale, true, 250, magnitude, 0..32768), // inverted to act as zoom
				KeyCode::Down     => update_value_i(&mut self.graph.scale, false, 250, magnitude, 0..32768), // inverted to act as zoom
				KeyCode::Right    => update_value_i(&mut self.graph.samples, true, 25, magnitude, 0..self.graph.width),
				KeyCode::Left     => update_value_i(&mut self.graph.samples, false, 25, magnitude, 0..self.graph.width),
				KeyCode::Char('q') => quit = true,
				KeyCode::Char(' ') => self.pause              = !self.pause,
				KeyCode::Char('s') => self.graph.scatter      = !self.graph.scatter,
				KeyCode::Char('h') => self.graph.show_ui      = !self.graph.show_ui,
				KeyCode::Char('r') => self.graph.references   = !self.graph.references,
				KeyCode::Tab => { // switch modes
					match self.mode {
						CurrentDisplayMode::Oscilloscope => self.mode = CurrentDisplayMode::Vectorscope,
						CurrentDisplayMode::Vectorscope => self.mode = CurrentDisplayMode::Spectroscope,
						CurrentDisplayMode::Spectroscope => self.mode = CurrentDisplayMode::Oscilloscope,
					}
				},
				_ => {},
			}
		};
	
		Ok(quit)
	}
}

pub fn update_value_f(val: &mut f64, base: f64, magnitude: f64, range: Range<f64>) {
	let delta = base * magnitude;
	if *val + delta > range.end {
		*val = range.end
	} else if *val + delta < range.start {
		*val = range.start
	} else {
		*val += delta;
	}
}

pub fn update_value_i(val: &mut u32, inc: bool, base: u32, magnitude: f64, range: Range<u32>) {
	let delta = (base as f64 * magnitude) as u32;
	if inc {
		if range.end - delta < *val {
			*val = range.end
		} else {
			*val += delta
		}
	} else if range.start + delta > *val {
		*val = range.start
	} else {
		*val -= delta
	}
}

impl From::<&crate::Args> for App {
	fn from(args: &crate::Args) -> Self {
		let graph = GraphConfig {
			axis_color: Color::DarkGray,
			labels_color: Color::Cyan,
			palette: vec![Color::Red, Color::Yellow, Color::Green, Color::Magenta],
			scale: args.range,
			width: args.buffer / (2 * args.channels as u32), // TODO also make bit depth customizable
			samples: args.buffer / (2 * args.channels as u32),
			references: !args.no_reference,
			show_ui: !args.no_ui,
			scatter: args.scatter,
			marker_type: if args.no_braille {
				Marker::Dot
			} else {
				Marker::Braille
			},
		};

		let oscilloscope = Oscilloscope {
			triggering: args.triggering,
			depth: args.check_depth,
			threshold: args.threshold,
			falling_edge: args.falling_edge,
			peaks: args.show_peaks,
		};

		let vectorscope = Vectorscope {};
		let spectroscope = Spectroscope {};

		App { 
			graph, oscilloscope, vectorscope, spectroscope,
			mode: CurrentDisplayMode::Oscilloscope,
			channels: args.channels,
			pause: false,
		}
	}
}

fn make_header<'a>(cfg: &GraphConfig, module_header: &'a str, fps: usize, pause: bool) -> Table<'a> {
	Table::new(
		vec![
			Row::new(
				vec![
					Cell::from("tui **scope").style(Style::default().fg(*cfg.palette.get(0).expect("empty palette?")).add_modifier(Modifier::BOLD)),
					Cell::from(module_header),
					Cell::from(format!("-{}+", cfg.scale)),
					Cell::from(format!("{}/{} spf", cfg.samples, cfg.width)),
					Cell::from(format!("{}fps", fps)),
					Cell::from(if cfg.scatter { "***" } else { "---" }),
					Cell::from(if pause { "||" } else { "|>" }),
				]
			)
		]
	)
	.style(Style::default().fg(cfg.labels_color))
	.widths(&[
		Constraint::Percentage(35),
		Constraint::Percentage(25),
		Constraint::Percentage(7),
		Constraint::Percentage(13),
		Constraint::Percentage(6),
		Constraint::Percentage(6),
		Constraint::Percentage(6)
	])
}
