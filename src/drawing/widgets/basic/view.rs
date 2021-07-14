use tui::{
    backend::Backend,
    layout::{Constraint, Rect},
    Frame,
};

use crate::drawing::{Axis, Border, Element, Event, EventStatus, Node, Padding, Widget};

/// A [`View`] widget displays its children in a one-dimensional array.
pub struct View<'a, B: Backend> {
    direction: Axis,
    children: Vec<Element<'a, B>>,
    padding: Padding,
    border: Border,
    width: Constraint,
    height: Constraint,
}

impl<'a, B: Backend> View<'a, B> {
    /// Creates a new [`View`] widget with no children.
    pub fn new(direction: Axis) -> Self {
        View::new_with_children(direction, vec![])
    }

    /// Creates a new [`View`] widget with the given [`Element`]s.
    pub fn new_with_children(direction: Axis, children: Vec<Element<'a, B>>) -> Self {
        Self {
            direction,
            children: children.into_iter().map(|c| c.into()).collect(),
            padding: Padding::Disabled,
            border: Border::Disabled,
            width: Constraint::Min(0),
            height: Constraint::Min(0),
        }
    }

    /// Sets the padding.
    pub fn padding(mut self, padding: Padding) -> Self {
        self.padding = padding;
        self
    }

    /// Sets the border.  Note that a border on a side takes up a single unit in terms of width/height.
    pub fn border(mut self, border: Border) -> Self {
        self.border = border;
        self
    }

    /// Pushes a new element onto the [`View`].
    pub fn push<E>(mut self, child: E) -> Self
    where
        E: Into<Element<'a, B>>,
    {
        self.children.push(child.into());
        self
    }

    /// Sets the width of the [`View`].
    pub fn width(mut self, width: Constraint) -> Self {
        self.width = width;
        self
    }

    /// Sets the height of the [`View`].
    pub fn height(mut self, height: Constraint) -> Self {
        self.height = height;
        self
    }
}

impl<'a, B: Backend> Widget<B> for View<'a, B> {
    // fn hash(&self, state: &mut Hasher) {
    //     self.direction.hash(state);
    //     self.padding.hash(state);
    //     self.border.hash(state);

    //     for child in &self.children {
    //         child.hash(state);
    //     }
    // }

    fn draw(&mut self, ctx: &mut Frame<'_, B>, node: &'_ Node) {
        self.children
            .iter_mut()
            .zip(node.children())
            .for_each(|(child, child_node)| {
                child.draw(ctx, child_node);
            });
    }

    fn layout(&self, bounds: Rect) -> Node {
        use tui::layout::{Direction, Layout};

        let desired_constraints: Vec<Constraint> = self
            .children
            .iter()
            .map(|child| match self.direction {
                Axis::Horizontal => child.width(),
                Axis::Vertical => child.height(),
            })
            .collect();

        let constrained_bounds = Layout::default()
            .constraints(desired_constraints)
            .direction(match self.direction {
                Axis::Horizontal => Direction::Horizontal,
                Axis::Vertical => Direction::Vertical,
            })
            .split(bounds);

        Node::new(
            bounds,
            self.children
                .iter()
                .zip(constrained_bounds)
                .map(|(child, bound)| child.layout(bound))
                .collect(),
        )
    }

    fn width(&self) -> Constraint {
        self.width
    }

    fn height(&self) -> Constraint {
        self.height
    }

    fn on_event(&mut self, event: Event) -> EventStatus {
        for child in &mut self.children {
            match child.on_event(event.clone()) {
                EventStatus::Handled => {
                    return EventStatus::Handled;
                }
                _ => {}
            }
        }

        EventStatus::Ignored
    }
}

impl<'a, B: 'a + Backend> From<View<'a, B>> for Element<'a, B> {
    fn from(view: View<'a, B>) -> Self {
        Element::new(Box::new(view))
    }
}
