use tui::{
    text::Span,
    widgets::{Block, Borders},
};

use crate::canvas::Painter;

/// A factory pattern builder for a tui [`Block`].
pub struct BlockBuilder {
    borders: Borders,
    selected: bool,
    expanded: bool,
    name: &'static str,
    extra_text: Option<String>,
}

impl BlockBuilder {
    /// Creates a new [`BlockBuilder`] with the name of block.
    pub fn new(name: &'static str) -> Self {
        Self {
            borders: Borders::ALL,
            selected: false,
            expanded: false,
            name,
            extra_text: None,
        }
    }

    /// Indicates that this block is currently selected, and should be drawn as such.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Indicates that this block is currently expanded, and should be drawn as such.
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }

    /// Indicates that this block has some extra text beyond the name.
    pub fn extra_text(mut self, extra_text: String) -> Self {
        self.extra_text = Some(extra_text);
        self
    }

    /// Determines the borders of the built [`Block`].
    pub fn borders(mut self, borders: Borders) -> Self {
        self.borders = borders;
        self
    }

    pub fn build<'a>(self, painter: &'a Painter) -> Block<'a> {
        let has_top_bottom_border =
            self.borders.contains(Borders::TOP) || self.borders.contains(Borders::BOTTOM);

        let block = Block::default()
            .border_style(if self.selected {
                painter.colours.highlighted_border_style
            } else {
                painter.colours.border_style
            })
            .borders(self.borders);

        if has_top_bottom_border {
            block.title(Span::styled(
                format!(" {} ", self.name),
                painter.colours.widget_title_style,
            ))
        } else {
            block
        }
    }
}
