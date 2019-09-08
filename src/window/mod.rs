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
		let size = f.size();
		Block::default().title("CPU Usage").borders(Borders::ALL).render(&mut f, size);
	})
}
