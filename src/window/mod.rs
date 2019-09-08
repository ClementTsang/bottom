use std::io;
use termion::raw::IntoRawMode;
use tui::{
	backend::TermionBackend,
	layout::{Constraint, Direction, Layout},
	widgets::{Block, Borders, Widget},
	Terminal,
};

pub fn create_terminal() -> Result<(), io::Error> {
	let stdout = io::stdout().into_raw_mode()?;
	let backend = TermionBackend::new(stdout);
	let mut terminal = Terminal::new(backend)?;
	terminal.clear()?;
	terminal.draw(|mut f| {
		let vertical_chunks = Layout::default()
			.direction(Direction::Vertical)
			.margin(1)
			.constraints([Constraint::Percentage(33), Constraint::Percentage(34), Constraint::Percentage(33)].as_ref())
			.split(f.size());
		let top_chunks = Layout::default()
			.direction(Direction::Horizontal)
			.margin(0)
			.constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
			.split(vertical_chunks[0]);
		let middle_chunks = Layout::default()
			.direction(Direction::Horizontal)
			.margin(0)
			.constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
			.split(vertical_chunks[1]);
		let middle_divided_chunk = Layout::default()
			.direction(Direction::Vertical)
			.margin(0)
			.constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
			.split(middle_chunks[0]);
		let bottom_chunks = Layout::default()
			.direction(Direction::Horizontal)
			.margin(0)
			.constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
			.split(vertical_chunks[2]);

		Block::default().title("CPU Usage").borders(Borders::ALL).render(&mut f, top_chunks[0]);
		Block::default().title("Memory Usage").borders(Borders::ALL).render(&mut f, top_chunks[1]);

		Block::default().title("Temperatures").borders(Borders::ALL).render(&mut f, middle_divided_chunk[0]);
		Block::default().title("Disk Usage").borders(Borders::ALL).render(&mut f, middle_divided_chunk[1]);
		Block::default().title("IO Usage").borders(Borders::ALL).render(&mut f, middle_chunks[1]);

		Block::default().title("Network").borders(Borders::ALL).render(&mut f, bottom_chunks[0]);
		Block::default().title("Processes").borders(Borders::ALL).render(&mut f, bottom_chunks[1]);
	})
}
