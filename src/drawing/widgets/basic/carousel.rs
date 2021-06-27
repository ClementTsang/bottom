use tui::{backend::Backend, layout::Constraint};

use crate::drawing::{Element, Widget};

/// Represents the state of a [`Carousel`].
pub struct State {
    /// This must never exceed the length of the elements.
    selected_index: usize,
}

pub struct NamedElement<'a, B>
where
    B: Backend,
{
    element: Element<'a, B>,
    name: &'a str,
}

/// A [`Carousel`] is a widget that shows only one of its children element at a time.
pub struct Carousel<'a, B>
where
    B: Backend,
{
    state: &'a mut State,
    children: Vec<NamedElement<'a, B>>,
    width: Constraint,
    height: Constraint,
}

impl<'a, B: Backend> Carousel<'a, B> {
    /// Creates a new [`Carousel`].
    pub fn new(state: &'a mut State, children: Vec<NamedElement<'a, B>>) -> Self {
        Self {
            state,
            children,
            width: Constraint::Min(0),
            height: Constraint::Min(0),
        }
    }

    /// Sets the width of the widget.
    pub fn width(mut self, width: Constraint) -> Self {
        self.width = width;
        self
    }

    /// Sets the height of the widget.
    pub fn height(mut self, height: Constraint) -> Self {
        self.height = height;
        self
    }
}

impl<'a, B: Backend> Widget<B> for Carousel<'a, B> {
    fn draw(&mut self, ctx: &mut tui::Frame<'_, B>, node: &'_ crate::drawing::Node) {
        // Draw arrows

        // Now draw the rest of the element...
    }

    fn layout(&self, bounds: tui::layout::Rect) -> crate::drawing::Node {
        todo!()
    }

    fn width(&self) -> tui::layout::Constraint {
        self.width
    }

    fn height(&self) -> tui::layout::Constraint {
        self.height
    }

    fn on_event(&mut self, event: crate::drawing::Event) -> crate::drawing::EventStatus {
        crate::drawing::EventStatus::Ignored
    }
}
