use ratatui::{
	style::Style,
	text::Span,
	widgets::{Axis, GraphType},
};

use crate::input::Matrix;

use super::{DataSet, Dimension, DisplayMode, GraphConfig};

#[derive(Default)]
pub struct Vectorscope {}

impl DisplayMode for Vectorscope {
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
			Dimension::X => ("left -", [-cfg.scale, cfg.scale]),
			Dimension::Y => ("| right", [-cfg.scale, cfg.scale]),
		};
		let mut a = Axis::default();
		if cfg.show_ui {
			// TODO don't make it necessary to check show_ui inside here
			a = a.title(Span::styled(name, Style::default().fg(cfg.labels_color)));
		}
		a.style(Style::default().fg(cfg.axis_color)).bounds(bounds)
	}

	fn references(&self, cfg: &GraphConfig) -> Vec<DataSet> {
		vec![
			DataSet::new(
				None,
				vec![(-cfg.scale, 0.0), (cfg.scale, 0.0)],
				cfg.marker_type,
				GraphType::Line,
				cfg.axis_color,
			),
			DataSet::new(
				None,
				vec![(0.0, -cfg.scale), (0.0, cfg.scale)],
				cfg.marker_type,
				GraphType::Line,
				cfg.axis_color,
			),
		]
	}

	fn process(&mut self, cfg: &GraphConfig, data: &Matrix<f64>) -> Vec<DataSet> {
		let mut out = Vec::new();

		for (n, chunk) in data.chunks(2).enumerate() {
			let mut tmp = vec![];
			match chunk.len() {
				2 => {
					for i in 0..std::cmp::min(chunk[0].len(), chunk[1].len()) {
						if i > cfg.samples as usize {
							break;
						}
						tmp.push((chunk[0][i], chunk[1][i]));
					}
				}
				1 => {
					for i in 0..chunk[0].len() {
						if i > cfg.samples as usize {
							break;
						}
						tmp.push((chunk[0][i], i as f64));
					}
				}
				_ => continue,
			}
			// split it in two for easier coloring
			// TODO configure splitting in multiple parts?
			let pivot = tmp.len() / 2;
			out.push(DataSet::new(
				Some(self.channel_name((n * 2) + 1)),
				tmp[pivot..].to_vec(),
				cfg.marker_type,
				if cfg.scatter {
					GraphType::Scatter
				} else {
					GraphType::Line
				},
				cfg.palette((n * 2) + 1),
			));
			out.push(DataSet::new(
				Some(self.channel_name(n * 2)),
				tmp[..pivot].to_vec(),
				cfg.marker_type,
				if cfg.scatter {
					GraphType::Scatter
				} else {
					GraphType::Line
				},
				cfg.palette(n * 2),
			));
		}

		out
	}
}
