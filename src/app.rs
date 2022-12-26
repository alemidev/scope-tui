use tui::{style::Color, widgets::GraphType, symbols};

// use crate::parser::SampleParser;

pub enum Dimension {
	X, Y
}

#[derive(Default)]
pub struct ChartNames {
	x: String,
	y: String,
}


pub struct ChartBounds {
	x: [f64;2],
	y: [f64;2],
}

impl Default for ChartBounds {
	fn default() -> Self {
		ChartBounds { x: [0.0, 0.0], y: [0.0, 0.0] }
	}
}

pub struct ChartReferences {
	pub x: Vec<(f64, f64)>,
	pub y: Vec<(f64, f64)>,
}

impl Default for ChartReferences {
	fn default() -> Self {
		ChartReferences {
			x: vec![(0.0, 0.0), (0.0, 1.0)],
			y: vec![(0.5, 1.0), (0.5, -1.0)]
		}
	}
}

pub struct AppConfig {
	pub title: String,
	pub primary_color: Color,
	pub secondary_color: Color,
	pub axis_color: Color,

	scale: u32,
	width: u32,
	vectorscope: bool,
	pub references: bool,

	pub marker_type: symbols::Marker,
	pub graph_type: GraphType,
}

pub struct App {
	pub cfg: AppConfig,
	pub references: ChartReferences,
	bounds: ChartBounds,
	names: ChartNames,
}

impl App {
	fn update_values(&mut self) {
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

	pub fn vectorscope(&self) -> bool {
		self.cfg.vectorscope
	}

	pub fn scale(&self) -> u32 {
		self.cfg.scale
	}

	pub fn width(&self) -> u32 {
		self.cfg.width
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

	pub fn graph_type(&self) -> GraphType {
		self.cfg.graph_type
	}

	pub fn set_vectorscope(&mut self, vectorscope: bool) {
		self.cfg.vectorscope = vectorscope;
		self.update_values();
	}

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
			primary_color: Color::Red,
			secondary_color: Color::Yellow,
			axis_color: Color::DarkGray,
			scale: args.range,
			width: args.buffer / 4, // TODO It's 4 because 2 channels and 2 bytes per sample!
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
