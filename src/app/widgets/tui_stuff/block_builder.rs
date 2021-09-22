use tui::{
    layout::Rect,
    text::Span,
    widgets::{Block, Borders},
};

use crate::canvas::Painter;

/// A factory pattern builder for a tui [`Block`].
pub struct BlockBuilder {
    borders: Borders,
    selected: bool,
    show_esc: bool,
    name: &'static str,
    hide_title: bool,
    extra_text: Option<String>,
}

impl BlockBuilder {
    /// Creates a new [`BlockBuilder`] with the name of block.
    pub fn new(name: &'static str) -> Self {
        Self {
            borders: Borders::ALL,
            selected: false,
            show_esc: false,
            name,
            hide_title: false,
            extra_text: None,
        }
    }

    /// Indicates that this block is currently selected, and should be drawn as such.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Indicates that this block should show esc, and should be drawn as such.
    pub fn show_esc(mut self, show_esc: bool) -> Self {
        self.show_esc = show_esc;
        self
    }

    /// Indicates that this block has some extra text beyond the name.
    pub fn extra_text(mut self, extra_text: Option<String>) -> Self {
        self.extra_text = extra_text;
        self
    }

    /// Determines the borders of the built [`Block`].
    pub fn borders(mut self, borders: Borders) -> Self {
        self.borders = borders;
        self
    }

    /// Forcibly hides the title of the built [`Block`].
    pub fn hide_title(mut self, hide_title: bool) -> Self {
        self.hide_title = hide_title;
        self
    }

    /// Converts the [`BlockBuilder`] into an actual [`Block`].
    pub fn build(self, painter: &Painter, area: Rect) -> Block<'static> {
        let has_title = !self.hide_title
            && (self.borders.contains(Borders::TOP) || self.borders.contains(Borders::BOTTOM));

        let border_style = if self.selected {
            painter.colours.highlighted_border_style
        } else {
            painter.colours.border_style
        };

        let block = Block::default()
            .border_style(border_style)
            .borders(self.borders);

        let inner_width = block.inner(area).width as usize;

        if has_title {
            let name = Span::styled(
                format!(" {} ", self.name),
                painter.colours.widget_title_style,
            );
            let mut title_len = name.width();
            let mut title = vec![name, Span::from(""), Span::from(""), Span::from("")];

            if self.show_esc {
                const EXPAND_TEXT: &str = " Esc to go back ";
                const EXPAND_TEXT_LEN: usize = EXPAND_TEXT.len();

                let expand_span = Span::styled(EXPAND_TEXT, border_style);

                if title_len + EXPAND_TEXT_LEN <= inner_width {
                    title_len += EXPAND_TEXT_LEN;
                    title[3] = expand_span;
                }
            }

            if let Some(extra_text) = self.extra_text {
                let extra_span = Span::styled(
                    format!("{} ", extra_text),
                    painter.colours.widget_title_style,
                );
                let width = extra_span.width();
                if title_len + width <= inner_width {
                    title_len += width;
                    title[1] = extra_span;
                }
            }

            if self.show_esc {
                let difference = inner_width.saturating_sub(title_len);
                title[2] = Span::styled("─".repeat(difference), border_style);
            }

            block.title(title)
        } else {
            block
        }
    }
}
