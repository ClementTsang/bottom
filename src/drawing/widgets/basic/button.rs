use tui::{
    backend::Backend, layout::Constraint, style::Style as TuiStyle, text::Text, widgets::Paragraph,
};

use crate::drawing::{Element, Event, EventStatus, Node, Widget};

/// The [`State`] of a [`Button`] mostly represents whether it is pressed or not.
#[derive(Default)]
pub struct State {
    is_pressed: bool,
}

/// The [`Style`] of a [`Button`] determines how it looks.
#[derive(Default)]
pub struct Style {
    not_pressed: TuiStyle,
    pressed: TuiStyle,
}

/// A [`Button`] is just a simple text element that supports "press" states.
pub struct Button<'a> {
    text: &'a str,
    state: &'a mut State,
    width: Constraint,
    height: Constraint,
    style: Style,
}

impl<'a> Button<'a> {
    /// Creates a new [`Button`].
    pub fn new(text: &'a str, state: &'a mut State) -> Self {
        Self {
            text,
            state,
            width: Constraint::Min(0),
            height: Constraint::Min(0),
            style: Style::default(),
        }
    }

    /// Sets the style of the [`Button`].
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Sets the width of the [`Button`].

    pub fn width(mut self, width: Constraint) -> Self {
        self.width = width;
        self
    }

    /// Sets the height of the [`Button`].

    pub fn height(mut self, height: Constraint) -> Self {
        self.height = height;
        self
    }
}

impl<'a, B> Widget<B> for Button<'a>
where
    B: Backend,
{
    fn draw(&mut self, ctx: &mut tui::Frame<'_, B>, node: &'_ crate::drawing::Node) {
        let style = if self.state.is_pressed {
            self.style.pressed
        } else {
            self.style.not_pressed
        };

        ctx.render_widget(
            Paragraph::new(Text::styled(self.text, style)),
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

    fn on_event(&mut self, event: Event) -> EventStatus {
        // Support the enter key and a left click action.  These both assume that the button is "in focus".
        //
        // Note that for the "Enter" event, we don't actually "change" the state - we cannot detect mouse up.
        // TODO: Send "signal" back out with handled case
        use crossterm::event::{KeyCode, MouseButton, MouseEvent};

        match event {
            Event::Mouse(event) => match event {
                MouseEvent::Down(MouseButton::Left, _, _, _) => {
                    self.state.is_pressed = true;
                    EventStatus::Handled
                }
                MouseEvent::Up(MouseButton::Left, _, _, _) => {
                    self.state.is_pressed = false;
                    EventStatus::Handled
                }
                _ => EventStatus::Ignored,
            },
            Event::Keyboard(event) => {
                if event.code == KeyCode::Enter {
                    EventStatus::Handled
                } else {
                    EventStatus::Ignored
                }
            }
        }
    }
}

impl<'a, B: Backend> From<Button<'a>> for Element<'a, B> {
    fn from(button: Button<'a>) -> Self {
        Element::new(Box::new(button))
    }
}
