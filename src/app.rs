use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::{
	backend::CrosstermBackend,
	layout::{Constraint, Rect},
	style::{Modifier, Style},
	symbols::Marker,
	widgets::{Cell, Chart, LegendPosition, Row, Table},
	Terminal,
};
use std::{
	io::{self, Stdout}, time::{Duration, Instant},
};

use crate::{
	display::{
		oscilloscope::Oscilloscope, spectroscope::Spectroscope, update_value_f, update_value_i,
		vectorscope::Vectorscope, Dimension, DisplayMode, GraphConfig,
	},
	input::{Source, SampleTape},
};

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum CurrentDisplayMode {
	Oscilloscope,
	Vectorscope,
	Spectroscope,
}

impl CurrentDisplayMode {
	pub fn switch(&self) -> Self {
		match self {
			Self::Oscilloscope => Self::Vectorscope,
			Self::Vectorscope => Self::Spectroscope,
			Self::Spectroscope => Self::Oscilloscope,
		}
	}

	pub fn switch_reverse(&self) -> Self {
		match self {
			Self::Oscilloscope => Self::Spectroscope,
			Self::Spectroscope => Self::Vectorscope,
			Self::Vectorscope => Self::Oscilloscope,
		}
	}
}

pub struct App {
	#[allow(unused)]
	channels: u8,
	graph: GraphConfig,
	oscilloscope: Oscilloscope,
	vectorscope: Vectorscope,
	spectroscope: Spectroscope,
	mode: CurrentDisplayMode,
	frame_time: Option<Duration>,
}

// TODO another way to build this that doesn't require getting cli args directly!!!
impl App {
	pub fn new(ui: &crate::cfg::UiOptions, source: &crate::cfg::SourceOptions) -> Self {
		let graph = GraphConfig {
			axis_color: ui.axis_color,
			labels_color: ui.labels_color,
			palette: ui.palette_color.clone(),
			scale: ui.scale as f64,
			chunks: source.chunks as u32,
			samples: source.buffer / source.channels as u32 / 2 * source.chunks as u32,
			initial_samples: source.buffer / source.channels as u32 / 2,
			sampling_rate: source.sample_rate,
			references: !ui.no_reference,
			show_ui: !ui.no_ui,
			scatter: ui.scatter,
			pause: false,
			marker_type: if ui.no_braille {
				Marker::Dot
			} else {
				Marker::Braille
			},
		};

		let oscilloscope = Oscilloscope::default();
		let vectorscope = Vectorscope::default();
		let spectroscope = Spectroscope::from(source);

		App {
			graph,
			oscilloscope,
			vectorscope,
			spectroscope,
			mode: ui.mode,
			channels: source.channels as u8,
			frame_time: ui.framerate.map(|x| std::time::Duration::from_nanos(1000000000 / x)),
		}
	}

	pub fn run(
		&mut self,
		mut source: Box<dyn Source>,
		mut terminal: Terminal<CrosstermBackend<Stdout>>,
	) -> Result<(), io::Error> {
		let mut fps = 0;
		let mut framerate = 0;
		let mut last_poll = Instant::now();
		let mut channels = SampleTape::default();
		let mut last_frame = std::time::Instant::now();

		let mut tape = crate::input::tape::Multi::new(self.graph.chunks as usize, self.channels as usize);

		loop {
			while event::poll(Duration::from_millis(0))? {
				// process all enqueued events
				let event = event::read()?;

				if self.process_events(event.clone())? {
					return Ok(());
				}
				self.current_display_mut().handle(event);
			}

			let chunk = source.recv().ok_or(io::Error::new(
				io::ErrorKind::BrokenPipe,
				"data source returned null",
			))?;

			tape.write(chunk);

			if let Some(frame_time) = self.frame_time && std::time::Instant::now() - last_frame < frame_time {
				continue
			}
			last_frame = std::time::Instant::now();

			if !self.graph.pause {
				channels = tape.read();
			}

			fps += 1;

			if last_poll.elapsed().as_secs() >= 1 {
				framerate = fps;
				fps = 0;
				last_poll = Instant::now();
			}

			{
				let mut datasets = Vec::new();
				let graph = self.graph.clone(); // TODO cheap fix...
				if self.graph.references {
					datasets.append(&mut self.current_display_mut().references(&graph));
				}
				datasets.append(&mut self.current_display_mut().process(&graph, &channels));
				terminal.draw(|f| {
					let mut size = f.area();
					if self.graph.show_ui {
						f.render_widget(
							make_header(
								&self.graph,
								&self.current_display().header(&self.graph),
								self.current_display().mode_str(),
								framerate,
								self.graph.pause,
							),
							Rect {
								x: size.x,
								y: size.y,
								width: size.width,
								height: 1,
							}, // a 1px line at the top
						);
						size.height -= 1;
						size.y += 1;
					}
					let chart = Chart::new(datasets.iter().map(|x| x.into()).collect())
						.x_axis(self.current_display().axis(&self.graph, Dimension::X)) // TODO allow to have axis sometimes?
						.y_axis(self.current_display().axis(&self.graph, Dimension::Y))
						.legend_position(match self.graph.show_ui {
							false => None,
							true => Some(LegendPosition::TopRight),
						});
					f.render_widget(chart, size)
				})?;
			}
		}
	}

	fn current_display_mut(&mut self) -> &mut dyn DisplayMode {
		match self.mode {
			CurrentDisplayMode::Oscilloscope => &mut self.oscilloscope as &mut dyn DisplayMode,
			CurrentDisplayMode::Vectorscope => &mut self.vectorscope as &mut dyn DisplayMode,
			CurrentDisplayMode::Spectroscope => &mut self.spectroscope as &mut dyn DisplayMode,
		}
	}

	fn current_display(&self) -> &dyn DisplayMode {
		match self.mode {
			CurrentDisplayMode::Oscilloscope => &self.oscilloscope as &dyn DisplayMode,
			CurrentDisplayMode::Vectorscope => &self.vectorscope as &dyn DisplayMode,
			CurrentDisplayMode::Spectroscope => &self.spectroscope as &dyn DisplayMode,
		}
	}

	fn process_events(&mut self, event: Event) -> Result<bool, io::Error> {
		let mut quit = false;
		if let Event::Key(key) = event {
			if let KeyModifiers::CONTROL = key.modifiers {
				match key.code {
					// mimic other programs shortcuts to quit, for user friendlyness
					KeyCode::Char('c') | KeyCode::Char('q') | KeyCode::Char('w') => quit = true,
					_ => {}
				}
			}
			let magnitude = match key.modifiers {
				KeyModifiers::SHIFT => 10.0,
				KeyModifiers::CONTROL => 5.0,
				KeyModifiers::ALT => 0.2,
				_ => 1.0,
			};
			match key.code {
				KeyCode::Up => update_value_f(&mut self.graph.scale, 0.01, magnitude, 0.0..10.0), // inverted to act as zoom
				KeyCode::Down => update_value_f(&mut self.graph.scale, -0.01, magnitude, 0.0..10.0), // inverted to act as zoom
				KeyCode::Right => update_value_i(
					&mut self.graph.samples,
					true,
					25,
					magnitude,
					0..self.graph.chunks * self.graph.initial_samples * 2,
				),
				KeyCode::Left => update_value_i(
					&mut self.graph.samples,
					false,
					25,
					magnitude,
					0..self.graph.chunks * self.graph.initial_samples * 2,
				),
				KeyCode::Char('q') => quit = true,
				KeyCode::Char(' ') => self.graph.pause = !self.graph.pause,
				KeyCode::Char('s') => self.graph.scatter = !self.graph.scatter,
				KeyCode::Char('h') => self.graph.show_ui = !self.graph.show_ui,
				KeyCode::Char('r') => self.graph.references = !self.graph.references,
				KeyCode::Tab => self.mode = self.mode.switch(),
				KeyCode::BackTab => self.mode = self.mode.switch_reverse(),
				KeyCode::Esc => {
					self.graph.samples = self.graph.chunks * self.graph.initial_samples;
					self.graph.scale = 1.;
				}
				_ => {}
			}
		};

		Ok(quit)
	}
}

// TODO can these be removed or merged somewhere else?

fn make_header<'a>(
	cfg: &GraphConfig,
	module_header: &'a str,
	kind_o_scope: &'static str,
	fps: usize,
	pause: bool,
) -> Table<'a> {
	Table::new(
		vec![Row::new(vec![
			Cell::from(format!("{}::scope-tui", kind_o_scope)).style(
				Style::default()
					.fg(*cfg.palette.first().expect("empty palette?"))
					.add_modifier(Modifier::BOLD),
			),
			Cell::from(module_header),
			Cell::from(format!("-{:.2}x+", cfg.scale)),
			Cell::from(format!("{}/{} spf", cfg.samples, cfg.initial_samples * cfg.chunks)),
			Cell::from(format!("{}fps", fps)),
			Cell::from(if cfg.scatter { "***" } else { "---" }),
			Cell::from(if pause { "||" } else { "|>" }),
		])],
		vec![
			Constraint::Percentage(35),
			Constraint::Percentage(25),
			Constraint::Percentage(7),
			Constraint::Percentage(13),
			Constraint::Percentage(6),
			Constraint::Percentage(6),
			Constraint::Percentage(6),
		],
	)
	.style(Style::default().fg(cfg.labels_color))
}
