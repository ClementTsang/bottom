use std::borrow::Cow;

use crossterm::event::MouseEvent;
use indextree::NodeId;
use tui::{
    backend::Backend,
    layout::{Constraint, Layout, Rect},
    text::{Span, Spans},
    widgets::Paragraph,
    Frame,
};

use crate::{
    app::{
        does_bound_intersect_coordinate, event::WidgetEventResult, Component, SelectableType,
        Widget,
    },
    canvas::Painter,
    options::layout_options::LayoutRule,
};

/// A container that "holds"" multiple [`BottomWidget`]s through their [`NodeId`]s.
#[derive(PartialEq, Eq)]
pub struct Carousel {
    index: usize,
    children: Vec<(NodeId, Cow<'static, str>)>,
    bounds: Rect,
    width: LayoutRule,
    height: LayoutRule,
    left_button_bounds: Rect,
    right_button_bounds: Rect,
}

impl Carousel {
    /// Creates a new [`Carousel`] with the specified children.
    pub fn new(children: Vec<(NodeId, Cow<'static, str>)>) -> Self {
        Self {
            index: 0,
            children,
            bounds: Default::default(),
            width: Default::default(),
            height: Default::default(),
            left_button_bounds: Default::default(),
            right_button_bounds: Default::default(),
        }
    }

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

    /// Adds a new child to a [`Carousel`].
    pub fn add_child(&mut self, child: NodeId, name: Cow<'static, str>) {
        self.children.push((child, name));
    }

    /// Returns the currently selected [`NodeId`] if possible.
    pub fn get_currently_selected(&self) -> Option<NodeId> {
        self.children.get(self.index).map(|i| i.0.clone())
    }

    fn get_next(&self) -> Option<&(NodeId, Cow<'static, str>)> {
        self.children.get(if self.index + 1 == self.children.len() {
            0
        } else {
            self.index + 1
        })
    }

    fn get_prev(&self) -> Option<&(NodeId, Cow<'static, str>)> {
        self.children.get(if self.index > 0 {
            self.index - 1
        } else {
            self.children.len().saturating_sub(1)
        })
    }

    fn increment_index(&mut self) {
        if self.index + 1 == self.children.len() {
            self.index = 0;
        } else {
            self.index += 1;
        }
    }

    fn decrement_index(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        } else {
            self.index = self.children.len().saturating_sub(1);
        }
    }

    /// Draws the [`Carousel`] arrows, and returns back the remaining [`Rect`] to draw the child with.
    pub fn draw_carousel<B: Backend>(
        &mut self, painter: &Painter, f: &mut Frame<'_, B>, area: Rect,
    ) -> Rect {
        const CONSTRAINTS: [Constraint; 2] = [Constraint::Length(1), Constraint::Min(0)];
        let split_area = Layout::default()
            .constraints(CONSTRAINTS)
            .direction(tui::layout::Direction::Vertical)
            .split(area);

        self.set_bounds(split_area[0]);

        if let Some((_prev_id, prev_element_name)) = self.get_prev() {
            let prev_arrow_text = Spans::from(Span::styled(
                format!("◄ {}", prev_element_name),
                painter.colours.text_style,
            ));

            self.left_button_bounds = Rect::new(
                split_area[0].x,
                split_area[0].y,
                prev_arrow_text.width() as u16,
                split_area[0].height,
            );

            f.render_widget(
                Paragraph::new(vec![prev_arrow_text]).alignment(tui::layout::Alignment::Left),
                split_area[0],
            );
        }

        if let Some((_next_id, next_element_name)) = self.get_next() {
            let next_arrow_text = Spans::from(Span::styled(
                format!("{} ►", next_element_name),
                painter.colours.text_style,
            ));

            let width = next_arrow_text.width() as u16;

            self.right_button_bounds = Rect::new(
                split_area[0].right().saturating_sub(width + 1),
                split_area[0].y,
                width,
                split_area[0].height,
            );

            f.render_widget(
                Paragraph::new(vec![next_arrow_text]).alignment(tui::layout::Alignment::Right),
                split_area[0],
            );
        }

        split_area[1]
    }
}

impl Component for Carousel {
    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, new_bounds: Rect) {
        self.bounds = new_bounds;
    }

    fn handle_mouse_event(&mut self, event: MouseEvent) -> WidgetEventResult {
        match event.kind {
            crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left) => {
                let x = event.column;
                let y = event.row;

                if does_bound_intersect_coordinate(x, y, self.left_button_bounds) {
                    self.decrement_index();
                    WidgetEventResult::Redraw
                } else if does_bound_intersect_coordinate(x, y, self.right_button_bounds) {
                    self.increment_index();
                    WidgetEventResult::Redraw
                } else {
                    WidgetEventResult::NoRedraw
                }
            }
            _ => WidgetEventResult::NoRedraw,
        }
    }
}

impl Widget for Carousel {
    fn get_pretty_name(&self) -> &'static str {
        "Carousel"
    }

    fn width(&self) -> LayoutRule {
        self.width
    }

    fn height(&self) -> LayoutRule {
        self.height
    }

    fn selectable_type(&self) -> SelectableType {
        if let Some(node) = self.get_currently_selected() {
            SelectableType::Redirect(node)
        } else {
            SelectableType::Unselectable
        }
    }
}
