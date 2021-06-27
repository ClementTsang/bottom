use tui::{backend::Backend, layout::Constraint, style::Style as TuiStyle, widgets::Paragraph};

use crate::drawing::{Element, Event, Node, Widget};

/// The [`Style`] of a [`Text`] determines how it looks.
#[derive(Default)]
pub struct Style {
    text: TuiStyle,
}

/// A [`Text`] is just a wrapper around tui-rs' text widgets.
pub struct Text<'a> {
    text: &'a str,
    width: Constraint,
    height: Constraint,
    style: Style,
}

impl<'a> Text<'a> {
    /// Creates a new [`Text`].
    pub fn new(text: &'a str) -> Self {
        Self {
            text,
            width: Constraint::Min(0),
            height: Constraint::Min(0),
            style: Style::default(),
        }
    }

    /// Sets the style of the [`Text`].
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Sets the width of the [`Text`].

    pub fn width(mut self, width: Constraint) -> Self {
        self.width = width;
        self
    }

    /// Sets the height of the [`Text`].

    pub fn height(mut self, height: Constraint) -> Self {
        self.height = height;
        self
    }
}

impl<'a, B> Widget<B> for Text<'a>
where
    B: Backend,
{
    fn draw(&mut self, ctx: &mut tui::Frame<'_, B>, node: &'_ crate::drawing::Node) {
        ctx.render_widget(
            Paragraph::new(tui::text::Text::styled(self.text, self.style.text)),
            node.bounds(),
        );
    }

    fn layout(&self, bounds: tui::layout::Rect) -> crate::drawing::Node {
        Node::new(bounds, vec![])
    }

    fn width(&self) -> tui::layout::Constraint {
        self.width
    }

    fn height(&self) -> tui::layout::Constraint {
        self.height
    }

    fn on_event(&mut self, event: Event) -> crate::drawing::EventStatus {
        // Support the enter key and a left click action.
        crate::drawing::EventStatus::Ignored
        // match event {
        //     Event::Mouse(event) => todo!(),
        //     Event::Keyboard(event) => todo!(),
        // }
    }
}

impl<'a, B: Backend> From<Text<'a>> for Element<'a, B> {
    fn from(text: Text<'a>) -> Self {
        Element::new(Box::new(text))
    }
}
