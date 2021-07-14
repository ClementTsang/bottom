use tui::{
    backend::Backend,
    layout::{Constraint, Rect},
};

use crate::drawing::{Axis, Button, Element, Event, Text, View, Widget};

use super::button;

/// The [`State`] of an [`Alert`] tracks which button is selected.
#[derive(Default)]
pub struct State {
    selected_button_index: usize,
    button_states: Vec<button::State>,
}

/// An [`Alert`] serves to be a dialog box with text and user-selectable options.
/// It does not support scrolling.
pub struct Alert<'a, B: Backend> {
    selected_button_index: &'a mut usize,
    body: Element<'a, B>,
}

impl<'a, B: Backend + 'a> Alert<'a, B> {
    pub fn new(state: &'a mut State, text: &'a str, buttons: Vec<&'a str>) -> Self {
        let State {
            selected_button_index,
            button_states,
        } = state;

        let button_view = View::new_with_children(
            Axis::Horizontal,
            buttons
                .into_iter()
                .zip(button_states)
                .map(|(s, button_state)| Button::new(s, button_state).into())
                .collect(),
        );

        // It's just composed of a [`View`] with two [`Button`]s and a [`Text`]!
        let children = vec![Text::new(text).into(), button_view.into()];
        // TODO: Spacing, size, etc.
        let body = View::new_with_children(Axis::Vertical, children).into();

        Self {
            selected_button_index,
            body,
        }
    }
}

impl<'a, B: Backend> Widget<B> for Alert<'a, B> {
    fn draw(&mut self, ctx: &mut tui::Frame<'_, B>, node: &'_ crate::drawing::Node) {
        self.body.draw(ctx, node);
    }

    fn layout(&self, bounds: Rect) -> crate::drawing::Node {
        self.body.layout(bounds)
    }

    fn width(&self) -> Constraint {
        self.body.width()
    }

    fn height(&self) -> Constraint {
        self.body.height()
    }
}
