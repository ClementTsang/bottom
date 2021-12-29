use tui::style::Style;

use crate::tuine::{
    self, block,
    text_table::{self, DataRow, SortType, TextTableProps},
    Block, Shortcut, StatefulComponent, TextTable, TmpComponent, ViewContext,
};

/// A set of styles for a [`SimpleTable`].
#[derive(Default)]
pub struct StyleSheet {
    pub text: Style,
    pub selected_text: Style,
    pub table_header: Style,
    pub border: Style,
}

/// A [`SimpleTable`] is a wrapper around a [`TextTable`] with basic shortcut support already added for:
/// - Skipping to the start/end of the table
/// - Scrolling up/down by a page
/// - Configurable sorting options
pub struct SimpleTable<Message> {
    inner: Block<Message, Shortcut<Message, TextTable<Message>>>,
}

impl<Message> SimpleTable<Message> {
    #[track_caller]
    pub fn build<C: Into<std::borrow::Cow<'static, str>>, R: Into<DataRow>>(
        ctx: &mut ViewContext<'_>, style: StyleSheet, columns: Vec<C>, data: Vec<R>,
    ) -> Self {
        let shortcut = Shortcut::with_child(TextTable::build(
            ctx,
            TextTableProps::new(columns)
                .rows(data)
                .default_sort(SortType::Ascending(1))
                .style(text_table::StyleSheet {
                    text: style.text,
                    selected_text: style.selected_text,
                    table_header: style.table_header,
                }),
        ));

        Self {
            inner: Block::with_child(shortcut).style(block::StyleSheet {
                border: style.border,
            }),
        }
    }
}

impl<Message> TmpComponent<Message> for SimpleTable<Message> {
    fn draw<Backend>(
        &mut self, state_ctx: &mut tuine::StateContext<'_>, draw_ctx: &tuine::DrawContext<'_>,
        frame: &mut tui::Frame<'_, Backend>,
    ) where
        Backend: tui::backend::Backend,
    {
        self.inner.draw(state_ctx, draw_ctx, frame);
    }

    fn on_event(
        &mut self, state_ctx: &mut tuine::StateContext<'_>, draw_ctx: &tuine::DrawContext<'_>,
        event: tuine::Event, messages: &mut Vec<Message>,
    ) -> tuine::Status {
        self.inner.on_event(state_ctx, draw_ctx, event, messages)
    }

    fn layout(&self, bounds: tuine::Bounds, node: &mut tuine::LayoutNode) -> tuine::Size {
        self.inner.layout(bounds, node)
    }
}
