/// A [`SelectableBlock`] is an extension around a [`Block`], that adds selection
/// indication logic and binds [`Event::Keyboard`] events to **always** be captured by
/// the children of the [`SelectableBlock`].
pub struct SelectableBlock {}
