use tui::widgets::TableState;

use super::{BaseTableWidget, Coordinate, HandleClick, HandleScroll, MouseButton};

#[derive(Debug)]
pub enum ScrollDirection {
    /// UP means scrolling up --- this usually DECREMENTS
    Up,
    /// DOWN means scrolling down --- this usually INCREMENTS
    Down,
}

impl Default for ScrollDirection {
    fn default() -> Self {
        ScrollDirection::Down
    }
}

pub trait ScrollableTable: BaseTableWidget {
    /// Gets the table state of the widget.
    fn get_table_state(&self) -> &TableState;

    /// Gets the table state of the widget.
    fn get_mut_table_state(&mut self) -> &mut TableState;

    /// Function to get the current position of the scroll cursor.
    fn get_current_scrollable_position(&self) -> usize;

    /// Function to get a mutable reference to the current position of the scroll cursor.
    fn get_mut_current_scrollable_position(&mut self) -> &mut usize;

    /// Function to get the current number of rows in a scrollable table.
    fn get_num_rows(&self) -> usize;

    /// Function to get the current scroll direction.
    fn get_current_scroll_direction(&self) -> &ScrollDirection;

    /// Function to get a mutable reference to the current scroll direction.
    fn get_mut_current_scroll_direction(&mut self) -> &mut ScrollDirection;

    /// Handles position movement.
    fn move_position(&mut self, amount_to_move: i64) {
        let num_rows = self.get_num_rows();
        let current_position = self.get_mut_current_scrollable_position();

        let new_position = *current_position as i64 + amount_to_move;
        if new_position >= 0 && new_position < num_rows as i64 {
            *current_position = new_position as usize;

            let current_direction = self.get_mut_current_scroll_direction();
            if amount_to_move.is_negative() {
                *current_direction = ScrollDirection::Up;
            } else {
                *current_direction = ScrollDirection::Down;
            }
        }
    }
}

impl<T> HandleScroll for T
where
    T: ScrollableTable,
{
    fn on_scroll_up(&mut self) {
        self.move_position(-1);
    }

    fn on_scroll_down(&mut self) {
        self.move_position(1);
    }
}

impl<T> HandleClick for T
where
    T: ScrollableTable,
{
    fn on_click(&mut self, button: MouseButton, click_coord: Coordinate) {
        if let MouseButton::Left = button {
            if let Some(widget_bounds) = self.get_widget_bounds() {
                let clicked_entry = click_coord.y - widget_bounds.top_left_corner.y;
                let offset =
                    1 + if self.is_drawing_vertical_borders() {
                        1
                    } else {
                        0
                    } + self.get_table_gap();

                if clicked_entry >= offset {
                    let offset_clicked_entry = clicked_entry - offset;
                    if let Some(visual_index) = self.get_table_state().selected() {
                        self.move_position(offset_clicked_entry as i64 - visual_index as i64);
                    }
                }
            }
        }
    }
}
