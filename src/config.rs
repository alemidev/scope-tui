use tui::style::Color;

// use crate::parser::SampleParser;

pub enum Dimension {
	X, Y
}

#[derive(Default)]
pub struct ChartNames {
	pub x: String,
	pub y: String,
}


pub struct ChartBounds {
	pub x: [f64;2],
	pub y: [f64;2],
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
	pub axis_color: Color,
	pub palette: Vec<Color>,

	pub scale: i32,
	pub width: u32,
	pub vectorscope: bool,
	pub references: bool,
	pub show_ui: bool,
	pub peaks: bool,

	pub triggering: bool,
	pub threshold: f64,
	pub depth: u32,
	pub falling_edge: bool,

	pub scatter: bool,
	pub braille: bool,

	pub pause: bool,
}
