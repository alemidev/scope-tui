use ratatui::{widgets::{Axis, GraphType}, style::Style, text::Span};

use super::{DisplayMode, GraphConfig, DataSet, Dimension};

#[derive(Default)]
pub struct Vectorscope {}

impl DisplayMode for Vectorscope {
	fn from_args(_args: &crate::ScopeArgs) -> Self {
		Vectorscope::default()
	}

	fn mode_str(&self) -> &'static str {
		"vector"
	}

	fn channel_name(&self, index: usize) -> String {
		format!("{}", index)
	}

	fn header(&self, _: &GraphConfig) -> String {
		"live".into()
	}

	fn axis(&self, cfg: &GraphConfig, dimension: Dimension) -> Axis {
		let (name, bounds) = match dimension {
			Dimension::X => ("left -", [-(cfg.scale as f64), cfg.scale as f64]),
			Dimension::Y => ("| right", [-(cfg.scale as f64), cfg.scale as f64]),
		};
		let mut a = Axis::default();
		if cfg.show_ui { // TODO don't make it necessary to check show_ui inside here
			a = a.title(Span::styled(name, Style::default().fg(cfg.labels_color)));
		}
		a.style(Style::default().fg(cfg.axis_color)).bounds(bounds)
	}

	fn references(&self, cfg: &GraphConfig) -> Vec<DataSet> {
		vec![
			DataSet::new("".into(), vec![(-(cfg.scale as f64), 0.0), (cfg.scale as f64, 0.0)], cfg.marker_type, GraphType::Line, cfg.axis_color), 
			DataSet::new("".into(), vec![(0.0, -(cfg.scale as f64)), (0.0, cfg.scale as f64)], cfg.marker_type, GraphType::Line, cfg.axis_color),
		]
	}

	fn process(&mut self, cfg: &GraphConfig, data: &Vec<Vec<f64>>) -> Vec<DataSet> {
		let mut out = Vec::new();

		for (n, chunk) in data.chunks(2).enumerate() {
			let mut tmp = vec![];
			match chunk.len() {
				2 => {
					for i in 0..std::cmp::min(chunk[0].len(), chunk[1].len()) {
						if i > cfg.samples as usize { break }
						tmp.push((chunk[0][i], chunk[1][i]));
					}
				},
				1 => {
					for i in 0..chunk[0].len() {
						if i > cfg.samples as usize { break }
						tmp.push((chunk[0][i], i as f64));
					}
				},
				_ => continue,
			}
			// split it in two for easier coloring
			// TODO configure splitting in multiple parts?
			let pivot = tmp.len() / 2;
			out.push(DataSet::new(
				self.channel_name((n * 2) + 1),
				tmp[pivot..].to_vec(),
				cfg.marker_type,
				if cfg.scatter { GraphType::Scatter } else { GraphType::Line },
				cfg.palette((n * 2) + 1),
			));
			out.push(DataSet::new(
				self.channel_name(n * 2),
				tmp[..pivot].to_vec(),
				cfg.marker_type,
				if cfg.scatter { GraphType::Scatter } else { GraphType::Line },
				cfg.palette(n * 2),
			));
		}

		out
	}
}
