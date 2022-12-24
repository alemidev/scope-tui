use tui::{style::Color, widgets::GraphType, symbols};

// use crate::parser::SampleParser;

pub enum Axis {
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
	graph_type: GraphType,

	bounds: ChartBounds,
	names: ChartNames,
}

impl AppConfig {
	fn update_values(&mut self) {
		if self.vectorscope {
			self.bounds.x = [-(self.scale as f64), self.scale as f64];
			self.bounds.y = [-(self.scale as f64), self.scale as f64];
			self.names.x = "- left".into();
			self.names.y = "| right".into();
		} else {
			// it makes no sense to show self.scale on the left but it's kinda nice
			self.names.x = "- time".into();
			self.names.y = "| amplitude".into();
			self.bounds.x = [0.0, self.width as f64];
			self.bounds.y = [-(self.scale as f64), self.scale as f64];
		}
	}

	pub fn vectorscope(&self) -> bool {
		self.vectorscope
	}

	pub fn scale(&self) -> u32 {
		self.scale
	}

	pub fn width(&self) -> u32 {
		self.width
	}

	pub fn scatter(&self) -> bool {
		match self.graph_type {
			GraphType::Scatter => true,
			_ => false,
		}
	}

	pub fn graph_type(&self) -> GraphType {
		self.graph_type
	}

	pub fn bounds(&self, axis: Axis) -> [f64;2] {
		match axis {
			Axis::X => self.bounds.x,
			Axis::Y => self.bounds.y,
		}
	}

	pub fn name(&self, axis: Axis) -> &str {
		match axis {
			Axis::X => self.names.x.as_str(),
			Axis::Y => self.names.y.as_str(),
		}
	}

	pub fn set_vectorscope(&mut self, vectorscope: bool) {
		self.vectorscope = vectorscope;
		self.update_values();
	}

	pub fn update_scale(&mut self, increment: i32) {
		self.scale = ((self.scale as i32) + increment) as u32;
		self.update_values();
	}

	pub fn set_scatter(&mut self, scatter: bool) {
		self.graph_type = if scatter { GraphType::Scatter } else { GraphType::Line };
	}
}


impl From::<crate::Args> for AppConfig {
	fn from(args: crate::Args) -> Self {
		let marker_type = if args.no_braille { symbols::Marker::Dot } else { symbols::Marker::Braille };
		let graph_type  = if args.scatter    { GraphType::Scatter   } else { GraphType::Line          };

		let mut cfg = AppConfig {
			title: "TUI Oscilloscope  --  <me@alemi.dev>".into(),
			primary_color: Color::Red,
			secondary_color: Color::Yellow,
			axis_color: Color::DarkGray,
			scale: args.scale,
			width: args.width / 4, // TODO It's 4 because 2 channels and 2 bytes per sample!
			vectorscope: args.vectorscope,
			references: !args.no_reference,
			bounds: ChartBounds::default(),
			names: ChartNames::default(),
			marker_type, graph_type,
		};

		cfg.update_values();

		cfg
	}
}
