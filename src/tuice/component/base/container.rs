use tui::{backend::Backend, layout::Rect, Frame};

use crate::tuice::{Bounds, Element, Event, LayoutNode, Size, Status, TmpComponent};

/// A [`Container`] just contains a child, as well as being able to be sized.
///
/// Inspired by Flutter's [Container class](https://api.flutter.dev/flutter/widgets/Container-class.html).
#[derive(Default)]
pub struct Container<'a, Message> {
    width: Option<u16>,
    height: Option<u16>,
    child: Option<Box<Element<'a, Message>>>,
}

impl<'a, Message> Container<'a, Message> {
    pub fn with_child(child: Element<'a, Message>) -> Self {
        Self {
            width: None,
            height: None,
            child: Some(child.into()),
        }
    }

    pub fn child(mut self, child: Option<Element<'a, Message>>) -> Self {
        self.child = child.map(|c| c.into());
        self
    }

    pub fn width(mut self, width: Option<u16>) -> Self {
        self.width = width;
        self
    }

    pub fn height(mut self, height: Option<u16>) -> Self {
        self.height = height;
        self
    }
}

impl<'a, Message> TmpComponent<Message> for Container<'a, Message> {
    fn draw<B>(&mut self, area: Rect, _frame: &mut Frame<'_, B>)
    where
        B: Backend,
    {
        todo!()
    }

    fn on_event(&mut self, _area: Rect, _event: Event, _messages: &mut Vec<Message>) -> Status {
        todo!()
    }

    fn layout(&self, bounds: Bounds, node: &mut LayoutNode) -> Size {
        let (width, height) = if let Some(child) = &self.child {
            let mut child_node = LayoutNode::default();

            fn bounds_if_exist(val: Option<u16>, min_bound: u16, max_bound: u16) -> (u16, u16) {
                if let Some(val) = val {
                    let val = val.clamp(min_bound, max_bound);
                    (val, val)
                } else {
                    (min_bound, max_bound)
                }
            }

            let child_bounds = {
                let (min_width, max_width) =
                    bounds_if_exist(self.width, bounds.min_width, bounds.max_width);
                let (min_height, max_height) =
                    bounds_if_exist(self.height, bounds.min_height, bounds.max_height);

                Bounds {
                    min_width,
                    min_height,
                    max_width,
                    max_height,
                }
            };

            let child_size = child.layout(child_bounds, &mut child_node);

            // Note that this is implicitly bounded by our above calculations,
            // no need to recheck if it's valid!
            (child_size.width, child_size.height)
        } else {
            (bounds.min_width, bounds.min_height)
        };

        Size { height, width }
    }
}
