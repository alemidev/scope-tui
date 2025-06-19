pub mod oscilloscope;
pub mod spectroscope;
pub mod vectorscope;

use crossterm::event::Event;
use ratatui::{
	style::{Color, Style},
	symbols::Marker,
	widgets::{Axis, Dataset, GraphType},
};

use crate::input::Matrix;

pub enum Dimension {
	X,
	Y,
}

#[derive(Debug, Clone, Default)]
pub struct GraphConfig {
	pub pause: bool,
	pub samples: u32,
	pub sampling_rate: u32,
	pub scale: f64,
	pub width: u32,
	pub scatter: bool,
	pub references: bool,
	pub show_ui: bool,
	pub marker_type: Marker,
	pub palette: Vec<Color>,
	pub labels_color: Color,
	pub axis_color: Color,
}

impl GraphConfig {
	pub fn palette(&self, index: usize) -> Color {
		*self
			.palette
			.get(index % self.palette.len())
			.unwrap_or(&Color::White)
	}
}

#[allow(clippy::ptr_arg)] // TODO temporarily! it's a shitty solution
pub trait DisplayMode {
	// MUST define
	fn axis(&self, cfg: &GraphConfig, dimension: Dimension) -> Axis; // TODO simplify this
	fn process(&mut self, cfg: &GraphConfig, data: &Matrix<f64>) -> Vec<DataSet>;
	fn mode_str(&self) -> &'static str;

	// SHOULD override
	fn channel_name(&self, index: usize) -> String {
		format!("{}", index)
	}
	fn header(&self, _cfg: &GraphConfig) -> String {
		"".into()
	}
	fn references(&self, _cfg: &GraphConfig) -> Vec<DataSet> {
		vec![]
	}
	fn handle(&mut self, _event: Event) {}
}

pub struct DataSet {
	name: Option<String>,
	data: Vec<(f64, f64)>,
	marker_type: Marker,
	graph_type: GraphType,
	color: Color,
}

impl<'a> From<&'a DataSet> for Dataset<'a> {
	fn from(ds: &'a DataSet) -> Dataset<'a> {
		let mut out = Dataset::default(); // TODO creating a binding is kinda ugly, is it avoidable?
		if let Some(name) = &ds.name {
			out = out.name(name.clone());
		}
		out.marker(ds.marker_type)
			.graph_type(ds.graph_type)
			.style(Style::default().fg(ds.color))
			.data(&ds.data)
	}
}

// TODO this is pretty ugly but I need datasets which own the data
impl DataSet {
	pub fn new(
		name: Option<String>,
		data: Vec<(f64, f64)>,
		marker_type: Marker,
		graph_type: GraphType,
		color: Color,
	) -> Self {
		DataSet {
			name,
			data,
			marker_type,
			graph_type,
			color,
		}
	}
}

pub(crate) fn update_value_f(
	val: &mut f64,
	base: f64,
	magnitude: f64,
	range: std::ops::Range<f64>,
) {
	let delta = base * magnitude;
	if *val + delta > range.end {
		*val = range.end
	} else if *val + delta < range.start {
		*val = range.start
	} else {
		*val += delta;
	}
}

pub(crate) fn update_value_i(
	val: &mut u32,
	inc: bool,
	base: u32,
	magnitude: f64,
	range: std::ops::Range<u32>,
) {
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
