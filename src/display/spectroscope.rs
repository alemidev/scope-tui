use ratatui::{widgets::{Axis, GraphType}, style::Style, text::Span};

use super::{DisplayMode, GraphConfig, DataSet, Dimension};

use easyfft::prelude::*;

pub struct Spectroscope {}

impl DisplayMode for Spectroscope {
	fn channel_name(&self, index: usize) -> String {
		format!("{}", index)
	}

	fn header(&self, _: &GraphConfig) -> String {
		"live".into()
	}

	fn axis(&self, cfg: &GraphConfig, dimension: Dimension) -> Axis {
		let (name, bounds) = match dimension {
			Dimension::X => ("frequency -", [-(cfg.scale as f64), cfg.scale as f64]),
			Dimension::Y => ("| level", [-(cfg.scale as f64), cfg.scale as f64]),
		};
		let mut a = Axis::default();
		if cfg.show_ui { // TODO don't make it necessary to check show_ui inside here
			a = a.title(Span::styled(name, Style::default().fg(cfg.labels_color)));
		}
		a.style(Style::default().fg(cfg.axis_color)).bounds(bounds)
	}

	fn references(&self, cfg: &GraphConfig) -> Vec<DataSet> {
		vec![
			DataSet::new("".into(), vec![(0.0, 0.0), (20000.0, 0.0)], cfg.marker_type, GraphType::Line, cfg.axis_color), 
			DataSet::new("".into(), vec![(0.0, 0.0), (0.0, cfg.scale as f64)], cfg.marker_type, GraphType::Line, cfg.axis_color),
		]
	}

	fn process(&self, cfg: &GraphConfig, data: &Vec<Vec<f64>>) -> Vec<DataSet> {
		let mut out = Vec::new();

		for (n, chunk) in data.iter().enumerate() {
			let tmp = chunk.real_fft().iter().map(|x| (x.re, x.im)).collect();
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
}
