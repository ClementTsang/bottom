//! Some code around handling events.

use std::sync::mpsc::Sender;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};

use crate::{
    app::{App, layout_manager::WidgetDirection},
    collection::Data,
};

/// Events sent to the main thread.
#[derive(Debug)]
pub enum BottomEvent {
    Resize,
    KeyInput(KeyEvent),
    MouseInput(MouseEvent),
    PasteEvent(String),
    Update(Box<Data>),
    Clean,
    Terminate,
}

/// Events sent to the collection thread.
#[derive(Debug)]
pub enum CollectionThreadEvent {
    Reset,
}

/// Handle a [`MouseEvent`].
pub fn handle_mouse_event(event: MouseEvent, app: &mut App) {
    match event.kind {
        MouseEventKind::ScrollUp => app.handle_scroll_up(),
        MouseEventKind::ScrollDown => app.handle_scroll_down(),
        MouseEventKind::Down(button) => {
            let (x, y) = (event.column, event.row);
            if !app.app_config_fields.disable_click {
                match button {
                    crossterm::event::MouseButton::Left => {
                        // Trigger left click widget activity
                        app.on_left_mouse_up(x, y);
                    }
                    crossterm::event::MouseButton::Right => {}
                    _ => {}
                }
            }
        }
        _ => {}
    };
}

/// Handle a [`KeyEvent`].
pub fn handle_key_event_or_break(
    event: KeyEvent, app: &mut App, reset_sender: &Sender<CollectionThreadEvent>,
) -> bool {
    // c_debug!("KeyEvent: {event:?}");

    if event.modifiers.is_empty() {
        // Required catch for searching - otherwise you couldn't search with q.
        if event.code == KeyCode::Char('q') && !app.is_in_search_widget() {
            return true;
        }
        match event.code {
            KeyCode::End => app.skip_to_last(),
            KeyCode::Home => app.skip_to_first(),
            KeyCode::Up => app.on_up_key(),
            KeyCode::Down => app.on_down_key(),
            KeyCode::Left => app.on_left_key(),
            KeyCode::Right => app.on_right_key(),
            KeyCode::Char(caught_char) => app.on_char_key(caught_char),
            KeyCode::Esc => app.on_esc(),
            KeyCode::Enter => app.on_enter(),
            KeyCode::Tab => app.on_tab(),
            KeyCode::Backspace => app.on_backspace(),
            KeyCode::Delete => app.on_delete(),
            KeyCode::F(1) => app.toggle_ignore_case(),
            KeyCode::F(2) => app.toggle_search_whole_word(),
            KeyCode::F(3) => app.toggle_search_regex(),
            KeyCode::F(5) => app.toggle_tree_mode(),
            KeyCode::F(6) => app.toggle_sort_menu(),
            KeyCode::F(9) => app.kill_current_process(),
            KeyCode::PageDown => app.on_page_down(),
            KeyCode::PageUp => app.on_page_up(),
            _ => {}
        }
    } else {
        // Otherwise, track the modifier as well...
        if let KeyModifiers::ALT = event.modifiers {
            match event.code {
                KeyCode::Char('c') | KeyCode::Char('C') => app.toggle_ignore_case(),
                KeyCode::Char('w') | KeyCode::Char('W') => app.toggle_search_whole_word(),
                KeyCode::Char('r') | KeyCode::Char('R') => app.toggle_search_regex(),
                KeyCode::Char('h') => app.on_left_key(),
                KeyCode::Char('l') => app.on_right_key(),
                _ => {}
            }
        } else if let KeyModifiers::CONTROL = event.modifiers {
            if event.code == KeyCode::Char('c') {
                return true;
            }

            match event.code {
                KeyCode::Char('f') => app.on_slash(),
                KeyCode::Left => app.move_widget_selection(&WidgetDirection::Left),
                KeyCode::Right => app.move_widget_selection(&WidgetDirection::Right),
                KeyCode::Up => app.move_widget_selection(&WidgetDirection::Up),
                KeyCode::Down => app.move_widget_selection(&WidgetDirection::Down),
                KeyCode::Char('r') => {
                    if reset_sender.send(CollectionThreadEvent::Reset).is_ok() {
                        app.reset();
                    }
                }
                KeyCode::Char('a') => app.skip_cursor_beginning(),
                KeyCode::Char('e') => app.skip_cursor_end(),
                KeyCode::Char('u') if app.is_in_search_widget() => app.clear_search(),
                KeyCode::Char('w') => app.clear_previous_word(),
                KeyCode::Char('h') => app.on_backspace(),
                KeyCode::Char('d') => app.scroll_half_page_down(),
                KeyCode::Char('u') => app.scroll_half_page_up(),
                // KeyCode::Char('j') => {}, // Move down
                // KeyCode::Char('k') => {}, // Move up
                // KeyCode::Char('h') => {}, // Move right
                // KeyCode::Char('l') => {}, // Move left
                // Can't do now, CTRL+BACKSPACE doesn't work and graphemes
                // are hard to iter while truncating last (eloquently).
                // KeyCode::Backspace => app.skip_word_backspace(),
                _ => {}
            }
        } else if let KeyModifiers::SHIFT = event.modifiers {
            match event.code {
                KeyCode::Left => app.move_widget_selection(&WidgetDirection::Left),
                KeyCode::Right => app.move_widget_selection(&WidgetDirection::Right),
                KeyCode::Up => app.move_widget_selection(&WidgetDirection::Up),
                KeyCode::Down => app.move_widget_selection(&WidgetDirection::Down),
                KeyCode::Char(caught_char) => app.on_char_key(caught_char),
                _ => {}
            }
        }
    }

    false
}
