
use std::{io::{self, ErrorKind}, time::{Duration, Instant}};
use tui::{
	style::Color, widgets::GraphType, symbols,
	backend::Backend,
	widgets::{Block, Chart, Axis, Dataset},
	Terminal, text::Span, style::{Style, Modifier}, layout::Alignment
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
