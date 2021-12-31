use tui::layout::Rect;

use crate::{
    app::{Component, SelectableType, Widget},
    options::layout_options::WidgetLayoutRule,
};

pub struct Empty {
    width: WidgetLayoutRule,
    height: WidgetLayoutRule,
}

impl Default for Empty {
    fn default() -> Self {
        Self {
            width: WidgetLayoutRule::default(),
            height: WidgetLayoutRule::default(),
        }
    }
}

impl Empty {
    /// Sets the width.
    pub fn width(mut self, width: WidgetLayoutRule) -> Self {
        self.width = width;
        self
    }

    /// Sets the height.
    pub fn height(mut self, height: WidgetLayoutRule) -> Self {
        self.height = height;
        self
    }
}

impl Component for Empty {
    fn bounds(&self) -> Rect {
        Rect::default()
    }

    fn set_bounds(&mut self, _new_bounds: Rect) {}
}

impl Widget for Empty {
    fn get_pretty_name(&self) -> &'static str {
        ""
    }

    fn width(&self) -> WidgetLayoutRule {
        self.width
    }

    fn height(&self) -> WidgetLayoutRule {
        self.height
    }

    fn selectable_type(&self) -> SelectableType {
        SelectableType::Unselectable
    }
}
