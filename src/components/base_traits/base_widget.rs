use crate::app::layout_manager::{BottomWidget, BottomWidgetType};

use super::{Drawable, HandleClick, HandleKeyInputs, HandleScroll};

#[derive(Default, Debug)]
pub struct Coordinate {
    pub x: u16,
    pub y: u16,
}

#[derive(Default, Debug)]
pub struct WidgetCorners {
    pub top_left_corner: Coordinate,
    pub bottom_right_corner: Coordinate,
}

pub trait BaseWidget: HandleClick + HandleScroll + HandleKeyInputs + Drawable {
    /// Get the widget bounds - returns the top left corner (TLC) and the bottom
    /// right corner (BRC).
    fn get_widget_bounds(&self) -> Option<WidgetCorners>;

    /// Get if a border is being drawn around this widget.
    fn is_drawing_borders(&self) -> bool {
        self.is_drawing_horizontal_borders() && self.is_drawing_vertical_borders()
    }

    /// Get if horizontal borders are being drawn around this widget.
    fn is_drawing_horizontal_borders(&self) -> bool;

    /// Get if vertical borders are being drawn around this widget.
    fn is_drawing_vertical_borders(&self) -> bool;

    /// Obtain the widget ID.
    fn get_widget_id(&self) -> u64 {
        self.get_bottom_widget_details().widget_id
    }

    /// Obtain the widget type.
    fn get_widget_type(&self) -> &BottomWidgetType {
        &self.get_bottom_widget_details().widget_type
    }

    /// Obtain the widget layout details.
    fn get_bottom_widget_details(&self) -> &BottomWidget;
}

pub trait BaseTableWidget: BaseWidget {
    fn get_table_gap(&self) -> u16;
}

pub trait BaseGraphWidget: BaseWidget {}
