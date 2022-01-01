use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tui::style::Style;

use crate::tuine::{
    self, block,
    shortcut::ShortcutProps,
    text_table::{self, DataRow, SortType, TextTableProps, TextTableState},
    Block, Event, Shortcut, StatefulComponent, Status, TextTable, TmpComponent, ViewContext,
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
        sort_index: usize,
    ) -> Self {
        let text_table = TextTable::build(
            ctx,
            TextTableProps::new(columns)
                .rows(data)
                .default_sort(SortType::Ascending(sort_index))
                .style(text_table::StyleSheet {
                    text: style.text,
                    selected_text: style.selected_text,
                    table_header: style.table_header,
                }),
        );
        let shortcut = Shortcut::build(
            ctx,
            ShortcutProps::with_child(text_table)
                .shortcut(
                    Event::Keyboard(KeyEvent::new(KeyCode::Char('G'), KeyModifiers::empty())),
                    Box::new(|t, s, _d, _e, _m| {
                        let state = s.mut_state::<TextTableState>(t.key);
                        state.scroll.jump_to_last();
                        Status::Captured
                    }),
                )
                .shortcut(
                    Event::Keyboard(KeyEvent::new(KeyCode::Char('G'), KeyModifiers::SHIFT)),
                    Box::new(|t, s, _d, _e, _m| {
                        let state = s.mut_state::<TextTableState>(t.key);
                        state.scroll.jump_to_last();
                        Status::Captured
                    }),
                )
                .multi_shortcut(
                    vec![
                        Event::Keyboard(KeyEvent::new(KeyCode::Char('g'), KeyModifiers::empty())),
                        Event::Keyboard(KeyEvent::new(KeyCode::Char('g'), KeyModifiers::empty())),
                    ],
                    Box::new(|t, s, _d, _e, _m| {
                        let state = s.mut_state::<TextTableState>(t.key);
                        state.scroll.jump_to_first();
                        Status::Captured
                    }),
                ),
        );

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
