pub mod oscilloscope;
pub mod vectorscope;
pub mod spectroscope;

use crossterm::event::Event;
use ratatui::{widgets::{Dataset, Axis, GraphType}, style::{Style, Color}, symbols::Marker};

pub enum Dimension {
	X, Y
}

pub struct GraphConfig {
	pub samples: u32,
	pub scale: u32,
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
		*self.palette.get(index % self.palette.len()).unwrap_or(&Color::White)
	}
}

#[allow(clippy::ptr_arg)] // TODO temporarily! it's a shitty solution
pub trait DisplayMode {
	// MUST define
	fn axis(&self, cfg: &GraphConfig, dimension: Dimension) -> Axis; // TODO simplify this
	fn process(&self, cfg: &GraphConfig, data: &Vec<Vec<f64>>) -> Vec<DataSet>;

	// SHOULD override
	fn handle(&mut self, _event: Event) {}
	fn channel_name(&self, index: usize) -> String { format!("{}", index) }
	fn header(&self, _cfg: &GraphConfig) -> String { "".into() }
	fn references(&self, cfg: &GraphConfig) -> Vec<DataSet> {
		let half_width = cfg.samples as f64 / 2.0;
		vec![
			DataSet::new("".into(), vec![(0.0, 0.0), (cfg.width as f64, 0.0)], cfg.marker_type, GraphType::Line, cfg.axis_color),
			DataSet::new("".into(), vec![(half_width, -(cfg.scale as f64)), (half_width, cfg.scale as f64)], cfg.marker_type, GraphType::Line, cfg.axis_color),

		]
	}
}

pub struct DataSet {
	name: String,
	data: Vec<(f64, f64)>,
	marker_type: Marker,
	graph_type: GraphType,
	color: Color,
}

impl<'a> From::<&'a DataSet> for Dataset<'a> {
	fn from(ds: &'a DataSet) -> Dataset<'a> {
		Dataset::default()
			.name(ds.name.clone())
			.marker(ds.marker_type)
			.graph_type(ds.graph_type)
			.style(Style::default().fg(ds.color))
			.data(&ds.data)
		}
}

impl DataSet {
	pub fn new(
		name: String,
		data: Vec<(f64, f64)>,
		marker_type: Marker,
		graph_type: GraphType,
		color: Color
	) -> Self {
		DataSet { name, data, marker_type, graph_type, color }
	}
}

