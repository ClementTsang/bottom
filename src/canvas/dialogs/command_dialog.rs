use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::Line,
    widgets::{Padding, Paragraph, StatefulWidget, Widget, Wrap},
};

use crate::{
    canvas::{
        components::scroll_bar::{ScrollBarArgs, dialog_scroll_bar_area, draw_scroll_bar_buffer},
        drawing_utils::dialog_block,
    },
    collection::processes::Pid,
    options::config::style::Styles,
};

pub struct CommandDialog<'a> {
    styles: &'a Styles,
}

impl<'a> CommandDialog<'a> {
    pub fn new(styles: &'a Styles) -> Self {
        Self { styles }
    }
}

impl StatefulWidget for CommandDialog<'_> {
    type State = CommandDialogState;

    fn render(self, area: Rect, buffer: &mut Buffer, state: &mut Self::State) {
        if !state.is_open() {
            return;
        }

        buffer.set_style(area, self.styles.general_widget_style);

        let block = dialog_block(self.styles.border_type, self.styles.border_style)
            .title_top(Line::styled(state.title(), self.styles.widget_title_style))
            .title_top(
                Line::styled(" Esc to close ", self.styles.widget_title_style).right_aligned(),
            )
            .padding(Padding::right(1));
        let inner = block.inner(area);
        let scroll = state.scroll;

        let paragraph = Paragraph::new(state.content())
            .style(self.styles.text_style)
            .wrap(Wrap { trim: false });
        let line_count = paragraph.line_count(inner.width);
        let max_scroll = line_count
            .saturating_sub(inner.height.into())
            .min(u16::MAX.into()) as u16;
        let scroll = scroll.min(max_scroll);

        paragraph
            .block(block)
            .scroll((scroll, 0))
            .render(area, buffer);

        if max_scroll > 0 {
            draw_scroll_bar_buffer(
                buffer,
                dialog_scroll_bar_area(area),
                ScrollBarArgs {
                    content_length: line_count,
                    viewport_length: inner.height.into(),
                    position: scroll.into(),
                    style: self.styles.text_style,
                },
            );
        }

        state.max_scroll = max_scroll;
        state.viewport_height = inner.height;
        state.scroll = scroll;
    }
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
            scroll: 0,
            max_scroll: 0,
            viewport_height: 0,
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

    pub fn title(&self) -> String {
        match &self.content {
            CommandDialogContent::Command { pid, .. } => {
                format!(" Command for PID {pid} ")
            }
            CommandDialogContent::Grouped => " Command unavailable ".into(),
            CommandDialogContent::Closed => unreachable!("closed dialogs are not rendered"),
        }
    }

    pub fn content(&self) -> &str {
        match &self.content {
            CommandDialogContent::Command { command, .. } => command,
            CommandDialogContent::Grouped => {
                "This row represents multiple processes. Press Esc, then Tab to ungroup them before viewing a command."
            }
            CommandDialogContent::Closed => unreachable!("closed dialogs are not rendered"),
        }
    }

    pub fn close(&mut self) {
        self.content = CommandDialogContent::Closed;
        self.scroll = 0;
        self.max_scroll = 0;
        self.viewport_height = 0;
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
    use ratatui::{buffer::Buffer, layout::Rect, widgets::StatefulWidget};

    use crate::options::config::style::Styles;

    use super::*;

    fn line(buffer: &Buffer, y: u16) -> String {
        (0..buffer.area.width)
            .map(|x| buffer[(x, y)].symbol())
            .collect()
    }

    fn line_without_scrollbar(buffer: &Buffer, y: u16) -> String {
        let mut chars: Vec<_> = line(buffer, y).chars().collect();
        chars[12] = ' ';
        chars.into_iter().collect()
    }

    #[test]
    fn command_view_scrolls_through_every_wrapped_line() {
        let area = Rect::new(0, 0, 14, 4);
        let mut buffer = Buffer::empty(area);
        let mut view =
            CommandDialogState::command(42, "one two three four five six seven eight".into());
        let styles = Styles::default();
        let dialog = CommandDialog::new(&styles);

        dialog.render(area, &mut buffer, &mut view);
        assert_eq!(line_without_scrollbar(&buffer, 1), "│one two     │");
        assert_eq!(line_without_scrollbar(&buffer, 2), "│three four  │");

        view.scroll_down();
        let mut buffer = Buffer::empty(area);
        CommandDialog::new(&Styles::default()).render(area, &mut buffer, &mut view);
        assert_eq!(line_without_scrollbar(&buffer, 1), "│three four  │");
        assert_eq!(line_without_scrollbar(&buffer, 2), "│five six    │");

        view.scroll_down();
        view.scroll_down();
        let mut buffer = Buffer::empty(area);
        CommandDialog::new(&Styles::default()).render(area, &mut buffer, &mut view);
        assert_eq!(line_without_scrollbar(&buffer, 1), "│five six    │");
        assert_eq!(line_without_scrollbar(&buffer, 2), "│seven eight │");
    }

    #[test]
    fn command_view_rewraps_and_clamps_scroll_after_resize() {
        let small = Rect::new(0, 0, 14, 4);
        let large = Rect::new(0, 0, 28, 6);
        let mut buffer = Buffer::empty(small);
        let mut view = CommandDialogState::command(42, "one two three four five six".into());
        let styles = Styles::default();

        CommandDialog::new(&styles).render(small, &mut buffer, &mut view);
        view.scroll_to_end();

        let mut buffer = Buffer::empty(large);
        CommandDialog::new(&styles).render(large, &mut buffer, &mut view);
        assert_eq!(line(&buffer, 1), "│one two three four five   │");
        assert_eq!(line(&buffer, 2), "│six                       │");
    }
}
