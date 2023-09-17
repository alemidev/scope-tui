use crossterm::event::{Event, KeyModifiers, KeyCode};
use ratatui::{widgets::{Axis, GraphType}, style::Style, text::Span};

use crate::app::update_value_f;

use super::{DisplayMode, GraphConfig, DataSet, Dimension};

#[derive(Default)]
pub struct Oscilloscope {
	pub triggering: bool,
	pub falling_edge: bool,
	pub threshold: f64,
	pub depth: u32,
	pub peaks: bool,
}

impl DisplayMode for Oscilloscope {
	fn channel_name(&self, index: usize) -> String {
		match index {
			0 => "L".into(),
			1 => "R".into(),
			_ => format!("{}", index),
		}
	}

	fn header(&self, _: &GraphConfig) -> String {
		if self.triggering {
			format!(
				"{} {:.0}{} trigger",
				if self.falling_edge { "v" } else { "^" },
				self.threshold,
				if self.depth > 1 { format!(":{}", self.depth) } else { "".into() },
			)
		} else {
			"live".into()
		}
	}

	fn axis(&self, cfg: &GraphConfig, dimension: Dimension) -> Axis {
		let (name, bounds) = match dimension {
			Dimension::X => ("time -", [0.0, cfg.samples as f64]),
			Dimension::Y => ("| amplitude", [-(cfg.scale as f64), cfg.scale as f64]),
		};
		let mut a = Axis::default();
		if cfg.show_ui { // TODO don't make it necessary to check show_ui inside here
			a = a.title(Span::styled(name, Style::default().fg(cfg.labels_color)));
		}
		a.style(Style::default().fg(cfg.axis_color)).bounds(bounds)
	}

	fn process(&mut self, cfg: &GraphConfig, data: &Vec<Vec<f64>>) -> Vec<DataSet> {
		let mut out = Vec::new();

		let mut trigger_offset = 0;
		if self.triggering {
			for i in 0..data[0].len() {
				if triggered(&data[0], i, self.threshold, self.depth, self.falling_edge) { // triggered
					break;
				} else {
					trigger_offset += 1;
				}
			}
		}

		if self.triggering {
			out.push(DataSet::new("T".into(), vec![(0.0, self.threshold)], cfg.marker_type, GraphType::Scatter, cfg.labels_color));
		}

		for (n, channel) in data.iter().enumerate().rev() {
			let (mut min, mut max) = (0.0, 0.0);
			let mut tmp = Vec::new();
			for (i, sample) in channel.iter().enumerate() {
				if *sample < min { min = *sample };
				if *sample > max { max = *sample };
				if i >= trigger_offset {
					tmp.push(((i - trigger_offset) as f64, *sample));
				}
			}

			if self.peaks {
				out.push(DataSet::new(
					"".into(),
					vec![(0.0, min), (0.0, max)],
					cfg.marker_type,
					GraphType::Scatter,
					cfg.palette(n)
				))
			}

			out.push(DataSet::new(
				self.channel_name(n),
				tmp,
				cfg.marker_type,
				if cfg.scatter { GraphType::Scatter } else { GraphType::Line },
				cfg.palette(n),
			));
		}

		out
	}

	fn handle(&mut self, event: Event) {
		if let Event::Key(key) = event {
			let magnitude = match key.modifiers {
				KeyModifiers::SHIFT => 10.0,
				KeyModifiers::CONTROL => 5.0,
				KeyModifiers::ALT => 0.2,
				_ => 1.0,
			};
			match key.code {
				KeyCode::PageUp   => update_value_f(&mut self.threshold, 250.0, magnitude, 0.0..32768.0),
				KeyCode::PageDown => update_value_f(&mut self.threshold, -250.0, magnitude, 0.0..32768.0),
				KeyCode::Char('t') => self.triggering   = !self.triggering,
				KeyCode::Char('e') => self.falling_edge = !self.falling_edge,
				KeyCode::Char('p') => self.peaks        = !self.peaks,
				KeyCode::Char('=') => self.depth       += 1,
				KeyCode::Char('-') => self.depth       -= 1,
				KeyCode::Char('+') => self.depth       += 10,
				KeyCode::Char('_') => self.depth       -= 10,
				_ => {}
			}
		}
	}
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
			true
		} else {
			false
		}
	} else if data[index] <= threshold {
		for i in 1..=depth as usize {
			if data[index+i] <= threshold {
				return false;
			}
		}
		true
	} else {
		false
	}
}
