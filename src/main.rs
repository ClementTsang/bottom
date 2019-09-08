use crossterm::{input, AlternateScreen, InputEvent, KeyEvent};
use std::{
	io::{self, stdin, stdout, Write},
	sync::mpsc,
	thread,
	time::Duration,
};
use tui::{
	backend::CrosstermBackend,
	layout::{Constraint, Direction, Layout},
	widgets::{Block, Borders, Widget},
	Terminal,
};

mod widgets;

enum Event<I> {
	Input(I),
	Tick,
}

#[tokio::main]
async fn main() -> Result<(), io::Error> {
	let screen = AlternateScreen::to_alternate(true)?;
	let backend = CrosstermBackend::with_alternate_screen(screen)?;
	let mut terminal = Terminal::new(backend)?;
	terminal.hide_cursor()?;
	// Setup input handling
	let (tx, rx) = mpsc::channel();
	{
		let tx = tx.clone();
		thread::spawn(move || {
			let input = input();
			let reader = input.read_sync();
			for event in reader {
				if let InputEvent::Keyboard(key) = event {
					if tx.send(Event::Input(key.clone())).is_err() {
						return;
					}
				}
			}
		});
	}
	{
		let tx = tx.clone();
		thread::spawn(move || {
			let tx = tx.clone();
			loop {
				tx.send(Event::Tick).unwrap();
				thread::sleep(Duration::from_millis(250));
			}
		});
	}

	let mut app : widgets::App = widgets::App::new("rustop");
	terminal.clear()?;

	loop {
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
		})?;

		// TODO: Ctrl-C?
		if let Ok(recv) = rx.recv() {
			match recv {
				Event::Input(event) => match event {
					KeyEvent::Char(c) => app.on_key(c),
					KeyEvent::Left => {}
					KeyEvent::Right => {}
					KeyEvent::Up => {}
					KeyEvent::Down => {}
					KeyEvent::Ctrl('c') => break,
					_ => {}
				},
				Event::Tick => {
					app.update_data();
				}
			}
			if app.should_quit {
				break;
			}
		}
	}

	Ok(())
}
