use crate::{
    app::AppConfig,
    canvas::Painter,
    data_conversion::ConvertedData,
    tuine::{
        Bounds, DrawContext, LayoutNode, SimpleTable, Size, StateContext, Status, TmpComponent,
        BuildContext,
    },
};

use super::{simple_table, AppWidget};

/// A [`DiskTable`] is a table displaying temperature data.
///
/// It wraps a [`SimpleTable`], with set columns and manages extracting data and styling.
pub struct DiskTable<Message> {
    inner: SimpleTable<Message>,
}

impl<Message> DiskTable<Message> {}

impl<Message> AppWidget for DiskTable<Message> {
    fn build(
        ctx: &mut BuildContext<'_>, painter: &Painter, config: &AppConfig,
        data: &mut ConvertedData<'_>,
    ) -> Self {
        let style = simple_table::StyleSheet {
            text: painter.colours.text_style,
            selected_text: painter.colours.currently_selected_text_style,
            table_header: painter.colours.table_header_style,
            border: painter.colours.border_style,
        };
        let rows = data.disk_table();

        Self {
            inner: SimpleTable::build(
                ctx,
                style,
                vec!["Disk", "Mount", "Used", "Free", "Total", "R/s", "W/s"],
                rows,
                0,
            ),
        }
    }
}

impl<Message> TmpComponent<Message> for DiskTable<Message> {
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
