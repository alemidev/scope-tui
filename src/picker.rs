use std::io::Write;

use crossterm::event;
use ratatui::{Terminal, backend::CrosstermBackend, layout::Constraint, style::{Style, Stylize}, widgets::{Block, Row, Table, TableState}};

pub struct Picker<'a> {
	state: TableState,
	rows: Vec<Row<'a>>,
	header: Row<'a>,
	devices: Vec<String>,
	constraints: Vec<Constraint>,
}

#[cfg(feature =  "cpal")]
impl<'a> Picker<'a> {
	pub fn new(header: Vec<String>, devices: Vec<Vec<String>>) -> Self {
		Self {
			state: TableState::default(),
			devices: devices.iter().filter_map(|d| Some(d.first()?.to_string())).collect(),
			constraints: header.iter().enumerate().map(|(i, _)| Constraint::Min(if i == 0 { 50 } else { 15 })).collect(),
			rows: devices.into_iter().map(Row::new).collect(),
			header: Row::new(header).white().on_black().bold(),
		}
	}

	pub fn run<W: Write>(&mut self, mut term: Terminal<CrosstermBackend<W>>) -> Option<String> {
    use ratatui::layout::Layout;

		loop {

			let block_outer = Block::bordered()
				.title_alignment(ratatui::layout::HorizontalAlignment::Right)
				.dark_gray();

			let block = Block::bordered()
				.title("available sources")
				.title_alignment(ratatui::layout::HorizontalAlignment::Right)
				.red();

			let table = Table::new(self.rows.clone(), self.constraints.clone())
				.block(block)
				.header(self.header.clone())
				.row_highlight_style(Style::new().white().on_red())
				.white();

			match term.draw(|frame| {

				let [_, area_v, _] = Layout::vertical([
					Constraint::Percentage(10),
					Constraint::Percentage(80),
					Constraint::Percentage(10),
				])
					.areas(frame.area());

				let [_, area_h, _] = Layout::horizontal([
					Constraint::Percentage(10),
					Constraint::Percentage(80),
					Constraint::Percentage(10),
				])
					.areas(area_v);

				frame.render_widget(block_outer, frame.area());
				frame.render_stateful_widget(table, area_h, &mut self.state);
			}) {
				Ok(_frame) => {},
				Err(_e) => {
					break;
				},
			}

			match event::read() {
				Err(_e) => break,
				Ok(ev) => if let event::Event::Key(key_event) = ev {
					match key_event.code {
						event::KeyCode::Up => self.state.select_previous(),
						event::KeyCode::Down => self.state.select_next(),
						event::KeyCode::Enter => break,
						event::KeyCode::Esc | event::KeyCode::Char('q') => {
							self.state.selected()?;
							self.state.select(None);
						},
						_ => {},
					}
				}
			}
		}

		self.devices.get(self.state.selected()?)
			.cloned()
	}
}
