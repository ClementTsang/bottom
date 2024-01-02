use super::Container;

/// A widget within bottom.
///
/// We use an enum to represent them to avoid dynamic dispatch.
pub enum BottomWidget {}

impl BottomWidget {}

/// An [`Element`] represents a node in the overall layout tree.
pub enum Element {
    BottomWidget(BottomWidget),
    Container(Container),
}

impl Element {}
