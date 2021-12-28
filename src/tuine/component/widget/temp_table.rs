use crate::{
    canvas::Painter,
    tuine::{
        Bounds, DataRow, DrawContext, LayoutNode, SimpleTable, Size, StateContext, Status,
        TmpComponent, ViewContext,
    },
};

use super::simple_table;

/// A [`TempTable`] is a table displaying temperature data.
///
/// It wraps a [`SimpleTable`], with set columns and manages extracting data and styling.
pub struct TempTable<Message> {
    inner: SimpleTable<Message>,
}

impl<Message> TempTable<Message> {
    pub fn build<R: Into<DataRow>>(
        ctx: &mut ViewContext<'_>, painter: &Painter, data: Vec<R>,
    ) -> Self {
        let style = simple_table::StyleSheet {
            text: painter.colours.text_style,
            selected_text: painter.colours.currently_selected_text_style,
            table_header: painter.colours.table_header_style,
            border: painter.colours.border_style,
        };

        Self {
            inner: SimpleTable::build(ctx, style, vec!["Sensor", "Temp"], data),
        }
    }
}

impl<Message> TmpComponent<Message> for TempTable<Message> {
    fn draw<Backend>(
        &mut self, state_ctx: &mut StateContext<'_>, draw_ctx: &DrawContext<'_>,
        frame: &mut tui::Frame<'_, Backend>,
    ) where
        Backend: tui::backend::Backend,
    {
        self.inner.draw(state_ctx, draw_ctx, frame);
    }

    fn on_event(
        &mut self, state_ctx: &mut StateContext<'_>, draw_ctx: &DrawContext<'_>,
        event: crate::tuine::Event, messages: &mut Vec<Message>,
    ) -> Status {
        self.inner.on_event(state_ctx, draw_ctx, event, messages)
    }

    fn layout(&self, bounds: Bounds, node: &mut LayoutNode) -> Size {
        self.inner.layout(bounds, node)
    }
}
