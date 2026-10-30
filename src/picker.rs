use std::io::Write;

use crossterm::event;
use ratatui::{Terminal, backend::CrosstermBackend, style::{Style, Stylize}, text::Line, widgets::{Block, List, Padding}};
use ratatui::widgets::ListState;

pub struct Picker {
	state: ListState,
	devices: Vec<String>,
}

#[cfg(feature =  "cpal")]
impl Picker {
	pub fn new(devices: Vec<String>) -> Self {
		Self { state: ListState::default(), devices }
	}

	pub fn run<W: Write>(&mut self, mut term: Terminal<CrosstermBackend<W>>) -> String {
		loop {
			let block = Block::bordered()
				.padding(Padding::uniform(2))
				.title("available sources")
				.title_alignment(ratatui::layout::HorizontalAlignment::Right)
				.dark_gray();

			let rows = self.devices
				.iter()
				.cloned()
				.map(Line::from)
				.collect::<Vec<Line>>();

			let list = List::new(rows)
				.block(block)
				.scroll_padding(2)
				.highlight_spacing(ratatui::widgets::HighlightSpacing::Never)
				.highlight_style(Style::new().white().on_red())
				.dark_gray();

			match term.draw(|frame| {
				frame.render_stateful_widget(list, frame.area(), &mut self.state);
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
						event::KeyCode::Esc => self.devices.push("asd".to_string()),
						event::KeyCode::Enter => break,
						_ => {},
					}
				}
			}
		}

		self.devices.get(self.state.selected().unwrap_or_default())
			.cloned()
			.unwrap_or_default()
	}
}
