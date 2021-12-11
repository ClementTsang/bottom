use tui::backend::Backend;

use crate::tuice::{Component, Length};

pub struct SizedBox<'a, Message, B>
where
    B: Backend,
{
    width: Length,
    height: Length,
    child: Box<dyn Component<Message, B> + 'a>,
}

impl<'a, Message, B> SizedBox<'a, Message, B>
where
    B: Backend,
{
    /// Creates a new [`SizedBox`] for a child component
    /// with a [`Length::Flex`] width and height.
    pub fn new(child: Box<dyn Component<Message, B> + 'a>) -> Self {
        Self {
            width: Length::Flex,
            height: Length::Flex,
            child,
        }
    }

    /// Creates a new [`SizedBox`] for a child component
    /// with a [`Length::Flex`] height.
    pub fn with_width(child: Box<dyn Component<Message, B> + 'a>, width: Length) -> Self {
        Self {
            width,
            height: Length::Flex,
            child,
        }
    }

    /// Creates a new [`SizedBox`] for a child component
    /// with a [`Length::Flex`] width.
    pub fn with_height(child: Box<dyn Component<Message, B> + 'a>, height: Length) -> Self {
        Self {
            width: Length::Flex,
            height,
            child,
        }
    }

    /// Sets the width of the [`SizedBox`].
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the height of the [`SizedBox`].
    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }
}
