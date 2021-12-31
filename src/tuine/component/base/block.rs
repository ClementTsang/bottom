use std::{borrow::Cow, marker::PhantomData};

use tui::{backend::Backend, layout::Rect, style::Style, widgets::Borders, Frame};

use crate::tuine::{
    Bounds, DrawContext, Event, LayoutNode, Size, StateContext, Status, TmpComponent,
};

/// A set of styles for a [`Block`].
#[derive(Clone, Debug, Default)]
pub struct StyleSheet {
    pub border: Style,
}

/// A [`Block`] is a widget that draws a border around a child [`Component`], as well as optional
/// titles.
pub struct Block<Message, Child>
where
    Child: TmpComponent<Message>,
{
    _pd: PhantomData<Message>,
    child: Option<Child>,
    borders: Borders,
    style_sheet: StyleSheet,
    left_text: Option<Cow<'static, str>>,
    right_text: Option<Cow<'static, str>>,
}

impl<Message, Child> Block<Message, Child>
where
    Child: TmpComponent<Message>,
{
    pub fn with_child(child: Child) -> Self {
        Self {
            _pd: Default::default(),
            child: Some(child),
            borders: Borders::all(),
            style_sheet: Default::default(),
            left_text: None,
            right_text: None,
        }
    }

    pub fn child(mut self, child: Option<Child>) -> Self {
        self.child = child;
        self
    }

    pub fn style(mut self, style: StyleSheet) -> Self {
        self.style_sheet = style;
        self
    }

    fn inner_rect(&self, original: Rect) -> Rect {
        let mut inner = original;

        if self.borders.intersects(Borders::LEFT) {
            inner.x = inner.x.saturating_add(1).min(inner.right());
            inner.width = inner.width.saturating_sub(1);
        }
        if self.borders.intersects(Borders::TOP)
            || self.left_text.is_some()
            || self.right_text.is_some()
        {
            inner.y = inner.y.saturating_add(1).min(inner.bottom());
            inner.height = inner.height.saturating_sub(1);
        }
        if self.borders.intersects(Borders::RIGHT) {
            inner.width = inner.width.saturating_sub(1);
        }
        if self.borders.intersects(Borders::BOTTOM) {
            inner.height = inner.height.saturating_sub(1);
        }
        inner
    }

    fn outer_size(&self, original: Size) -> Size {
        let mut outer = original;

        if self.borders.intersects(Borders::LEFT) {
            outer.width = outer.width.saturating_add(1);
        }
        if self.borders.intersects(Borders::TOP)
            || self.left_text.is_some()
            || self.right_text.is_some()
        {
            outer.height = outer.height.saturating_add(1);
        }
        if self.borders.intersects(Borders::RIGHT) {
            outer.width = outer.width.saturating_add(1);
        }
        if self.borders.intersects(Borders::BOTTOM) {
            outer.height = outer.height.saturating_add(1);
        }

        outer
    }
}

impl<Message, Child> TmpComponent<Message> for Block<Message, Child>
where
    Child: TmpComponent<Message>,
{
    fn draw<B>(
        &mut self, state_ctx: &mut StateContext<'_>, draw_ctx: &DrawContext<'_>,
        frame: &mut Frame<'_, B>,
    ) where
        B: Backend,
    {
        let rect = draw_ctx.global_rect();

        frame.render_widget(
            tui::widgets::Block::default()
                .borders(self.borders)
                .border_style(self.style_sheet.border),
            rect,
        );

        if let Some(child) = &mut self.child {
            if let Some(child_draw_ctx) = draw_ctx.children().next() {
                child.draw(state_ctx, &child_draw_ctx, frame)
            }
        }
    }

    fn on_event(
        &mut self, state_ctx: &mut StateContext<'_>, draw_ctx: &DrawContext<'_>, event: Event,
        messages: &mut Vec<Message>,
    ) -> Status {
        if let Some(child_draw_ctx) = draw_ctx.children().next() {
            if let Some(child) = &mut self.child {
                return child.on_event(state_ctx, &child_draw_ctx, event, messages);
            }
        }

        Status::Ignored
    }

    fn layout(&self, bounds: Bounds, node: &mut LayoutNode) -> crate::tuine::Size {
        if let Some(child) = &self.child {
            // Reduce bounds based on borders
            let inner_rect = self.inner_rect(Rect::new(0, 0, bounds.max_width, bounds.max_height));
            let child_bounds = Bounds {
                min_width: bounds.min_width,
                min_height: bounds.min_height,
                max_width: inner_rect.width,
                max_height: inner_rect.height,
            };

            let mut child_node = LayoutNode::default();
            let child_size = child.layout(child_bounds, &mut child_node);

            child_node.rect = Rect::new(
                inner_rect.x,
                inner_rect.y,
                child_size.width,
                child_size.height,
            );
            node.children = vec![child_node];

            self.outer_size(child_size)
        } else {
            Size {
                width: 0,
                height: 0,
            }
        }
    }
}
