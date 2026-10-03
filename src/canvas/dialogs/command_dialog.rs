use ratatui::{
    Frame,
    layout::Rect,
    text::Line,
    widgets::{Padding, Paragraph, Wrap},
};

use crate::{
    canvas::{
        components::scroll_bar::{ScrollBarArgs, dialog_scroll_bar_area, draw_scroll_bar},
        drawing_utils::dialog_block,
    },
    collection::processes::Pid,
    options::config::style::Styles,
};

pub fn draw_command_dialog(
    f: &mut Frame<'_>, area: Rect, styles: &Styles, state: &mut CommandDialogState,
) {
    f.buffer_mut().set_style(area, styles.general_widget_style);

    let (title, content) = match &state.content {
        CommandDialogContent::Closed => unreachable!("closed dialogs are not rendered"),
        CommandDialogContent::Command { pid, command } => {
            (format!(" Command for PID {pid} "), command.as_str())
        }
        CommandDialogContent::Grouped => (
            " Command unavailable ".into(),
            "This row is grouped. Press Esc, then Tab to ungroup processes before viewing a command.",
        ),
    };
    let block = dialog_block(styles.border_type, styles.border_style)
        .title_top(Line::styled(title, styles.widget_title_style))
        .title_top(Line::styled(" Esc to close ", styles.widget_title_style).right_aligned())
        .padding(Padding::right(1));
    let inner = block.inner(area);
    let scroll = state.scroll;

    let paragraph = Paragraph::new(content)
        .style(styles.text_style)
        .wrap(Wrap { trim: false });
    let line_count = paragraph.line_count(inner.width);
    let max_scroll = line_count
        .saturating_sub(inner.height.into())
        .min(u16::MAX.into()) as u16;
    let scroll = scroll.min(max_scroll);

    f.render_widget(paragraph.block(block).scroll((scroll, 0)), area);

    if max_scroll > 0 {
        draw_scroll_bar(
            f,
            dialog_scroll_bar_area(area),
            ScrollBarArgs {
                // Each possible top line is one scrollbar position.
                content_length: usize::from(max_scroll) + 1,
                viewport_length: 1,
                position: scroll.into(),
                style: styles.text_style,
            },
        );
    }

    state.max_scroll = max_scroll;
    state.viewport_height = inner.height;
    state.scroll = scroll;
}

#[derive(Default)]
pub struct CommandDialogState {
    content: CommandDialogContent,
    scroll: u16,
    max_scroll: u16,
    viewport_height: u16,
}

#[derive(Default)]
enum CommandDialogContent {
    #[default]
    Closed,
    Command {
        pid: Pid,
        command: String,
    },
    Grouped,
}

impl CommandDialogState {
    pub fn command(pid: Pid, command: String) -> Self {
        Self {
            content: CommandDialogContent::Command { pid, command },
            ..Self::default()
        }
    }

    pub fn grouped() -> Self {
        Self {
            content: CommandDialogContent::Grouped,
            ..Self::default()
        }
    }

    pub fn is_open(&self) -> bool {
        !matches!(self.content, CommandDialogContent::Closed)
    }

    pub fn close(&mut self) {
        self.content = CommandDialogContent::Closed;
    }

    pub fn scroll_down(&mut self) {
        self.scroll_by(self.scroll.saturating_add(1));
    }

    pub fn scroll_up(&mut self) {
        self.scroll_by(self.scroll.saturating_sub(1));
    }

    pub fn page_down(&mut self) {
        self.scroll_by(self.scroll.saturating_add(self.viewport_height));
    }

    pub fn page_up(&mut self) {
        self.scroll_by(self.scroll.saturating_sub(self.viewport_height));
    }

    pub fn half_page_down(&mut self) {
        self.scroll_by(self.scroll.saturating_add(self.viewport_height / 2));
    }

    pub fn half_page_up(&mut self) {
        self.scroll_by(self.scroll.saturating_sub(self.viewport_height / 2));
    }

    pub fn scroll_to_start(&mut self) {
        self.scroll = 0;
    }

    pub fn scroll_to_end(&mut self) {
        self.scroll = self.max_scroll;
    }

    fn scroll_by(&mut self, position: u16) {
        self.scroll = position.min(self.max_scroll);
    }
}

#[cfg(test)]
mod tests {
    use ratatui::{Terminal, backend::TestBackend};

    use crate::options::config::style::Styles;

    use super::*;

    #[test]
    fn scrollbar_reaches_bottom_when_command_is_scrolled_to_end() {
        let mut terminal = Terminal::new(TestBackend::new(14, 4)).unwrap();
        let mut view = CommandDialogState::command(
            42,
            "one two three four five six seven eight nine ten eleven twelve".into(),
        );
        let styles = Styles::default();

        terminal
            .draw(|f| draw_command_dialog(f, f.area(), &styles, &mut view))
            .unwrap();
        view.scroll_to_end();
        terminal
            .draw(|f| draw_command_dialog(f, f.area(), &styles, &mut view))
            .unwrap();
        assert_eq!(terminal.backend().buffer()[(12, 2)].symbol(), "█");
    }
}
