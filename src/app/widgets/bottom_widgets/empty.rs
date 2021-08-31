use tui::layout::Rect;

use crate::{
    app::{Component, Widget},
    options::layout_options::LayoutRule,
};

pub struct Empty {
    width: LayoutRule,
    height: LayoutRule,
}

impl Default for Empty {
    fn default() -> Self {
        Self {
            width: LayoutRule::default(),
            height: LayoutRule::default(),
        }
    }
}

impl Empty {
    /// Sets the width.
    pub fn width(mut self, width: LayoutRule) -> Self {
        self.width = width;
        self
    }

    /// Sets the height.
    pub fn height(mut self, height: LayoutRule) -> Self {
        self.height = height;
        self
    }
}

impl Component for Empty {
    fn bounds(&self) -> Rect {
        // TODO: Maybe think of how to store this without making it available for clicking. Separate bounds out to the layout? Might
        // need to keep the bounds calculations for some components, so maybe implement it specifically for them.
        Rect::default()
    }

    fn set_bounds(&mut self, _new_bounds: Rect) {}
}

impl Widget for Empty {
    fn get_pretty_name(&self) -> &'static str {
        ""
    }

    fn width(&self) -> LayoutRule {
        self.width
    }

    fn height(&self) -> LayoutRule {
        self.height
    }
}
