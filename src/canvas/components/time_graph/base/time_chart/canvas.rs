//! Vendored from <https://github.com/fdehau/tui-rs/blob/fafad6c96109610825aad89c4bba5253e01101ed/src/widgets/canvas/mod.rs>
//! and <https://github.com/ratatui/ratatui/blob/65c520245aa20e99e64d9ffcb2062a4502a699ea/ratatui-widgets/src/canvas.rs>.
//!
//! The main thing this is pulled in for is overriding how `BrailleGrid`'s draw
//! logic works, as changing it is needed in order to draw all datasets in only
//! one layer back in [`super::TimeChart::render`]. More specifically,
//! the current implementation in ratatui `|=`s all the cells together if they
//! overlap, but since we are smashing all the layers together which may have
//! different colours, we instead just _replace_ whatever was in that cell
//! with the newer colour + character.
//!
//! See <https://github.com/ClementTsang/bottom/pull/918> and <https://github.com/ClementTsang/bottom/pull/937> for the
//! original motivation.

use super::grid::{CharGrid, Grid, HalfBlockGrid, PatternGrid};
use ratatui_core::symbols::braille::BRAILLE;
use ratatui_core::symbols::pixel::{OCTANTS, QUADRANTS, SEXTANTS};
use tui::prelude::BlockExt;
use tui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    symbols::Marker,
    text::Line,
    widgets::{
        Block, Widget,
        canvas::{Line as CanvasLine, Points},
    },
};

/// Interface for all shapes that may be drawn on a Canvas widget.
pub trait Shape {
    fn draw(&self, painter: &mut Painter<'_, '_>);
}

impl Shape for CanvasLine {
    fn draw(&self, painter: &mut Painter<'_, '_>) {
        let (x1, y1) = match painter.get_point(self.x1, self.y1) {
            Some(c) => c,
            None => return,
        };
        let (x2, y2) = match painter.get_point(self.x2, self.y2) {
            Some(c) => c,
            None => return,
        };
        let (dx, x_range) = if x2 >= x1 {
            (x2 - x1, x1..=x2)
        } else {
            (x1 - x2, x2..=x1)
        };
        let (dy, y_range) = if y2 >= y1 {
            (y2 - y1, y1..=y2)
        } else {
            (y1 - y2, y2..=y1)
        };

        if dx == 0 {
            for y in y_range {
                painter.paint(x1, y, self.color);
            }
        } else if dy == 0 {
            for x in x_range {
                painter.paint(x, y1, self.color);
            }
        } else if dy < dx {
            if x1 > x2 {
                draw_line_low(painter, x2, y2, x1, y1, self.color);
            } else {
                draw_line_low(painter, x1, y1, x2, y2, self.color);
            }
        } else if y1 > y2 {
            draw_line_high(painter, x2, y2, x1, y1, self.color);
        } else {
            draw_line_high(painter, x1, y1, x2, y2, self.color);
        }
    }
}

fn draw_line_low(
    painter: &mut Painter<'_, '_>, x1: usize, y1: usize, x2: usize, y2: usize, color: Color,
) {
    let dx = (x2 - x1) as isize;
    let dy = (y2 as isize - y1 as isize).abs();
    let mut d = 2 * dy - dx;
    let mut y = y1;
    for x in x1..=x2 {
        painter.paint(x, y, color);
        if d > 0 {
            y = if y1 > y2 {
                y.saturating_sub(1)
            } else {
                y.saturating_add(1)
            };
            d -= 2 * dx;
        }
        d += 2 * dy;
    }
}

fn draw_line_high(
    painter: &mut Painter<'_, '_>, x1: usize, y1: usize, x2: usize, y2: usize, color: Color,
) {
    let dx = (x2 as isize - x1 as isize).abs();
    let dy = (y2 - y1) as isize;
    let mut d = 2 * dx - dy;
    let mut x = x1;
    for y in y1..=y2 {
        painter.paint(x, y, color);
        if d > 0 {
            x = if x1 > x2 {
                x.saturating_sub(1)
            } else {
                x.saturating_add(1)
            };
            d -= 2 * dy;
        }
        d += 2 * dx;
    }
}

impl Shape for Points<'_> {
    fn draw(&self, painter: &mut Painter<'_, '_>) {
        for (x, y) in self.coords {
            if let Some((x, y)) = painter.get_point(*x, *y) {
                painter.paint(x, y, self.color);
            }
        }
    }
}

/// Label to draw some text on the canvas
#[derive(Debug, Clone)]
pub struct Label<'a> {
    x: f64,
    y: f64,
    spans: Line<'a>,
}

#[derive(Debug)]
pub struct Painter<'a, 'b> {
    context: &'a mut Context<'b>,
    resolution: (f64, f64),
}

impl Painter<'_, '_> {
    /// Convert the (x, y) coordinates to location of a point on the grid.
    pub fn get_point(&self, x: f64, y: f64) -> Option<(usize, usize)> {
        let [left, right] = self.context.x_bounds;
        let [bottom, top] = self.context.y_bounds;
        if x < left || x > right || y < bottom || y > top {
            return None;
        }
        let width = right - left;
        let height = top - bottom;
        if width <= 0.0 || height <= 0.0 {
            return None;
        }
        let x = ((x - left) * (self.resolution.0 - 1.0) / width).round() as usize;
        let y = ((top - y) * (self.resolution.1 - 1.0) / height).round() as usize;
        Some((x, y))
    }

    /// Paint a point of the grid.
    pub fn paint(&mut self, x: usize, y: usize, color: Color) {
        self.context.grid.paint(x, y, color);
    }
}

impl<'a, 'b> From<&'a mut Context<'b>> for Painter<'a, 'b> {
    fn from(context: &'a mut Context<'b>) -> Painter<'a, 'b> {
        let resolution = context.grid.resolution();
        Painter {
            context,
            resolution,
        }
    }
}

/// Holds the state of the Canvas when painting to it.
#[derive(Debug)]
pub struct Context<'a> {
    x_bounds: [f64; 2],
    y_bounds: [f64; 2],
    grid: Box<dyn Grid>,
    dirty: bool,
    labels: Vec<Label<'a>>,
}

impl<'a> Context<'a> {
    pub fn new(
        width: u16, height: u16, x_bounds: [f64; 2], y_bounds: [f64; 2], marker: Marker,
    ) -> Context<'a> {
        let grid = Self::marker_to_grid(width, height, marker);

        Context {
            x_bounds,
            y_bounds,
            grid,
            dirty: false,
            labels: Vec::new(),
        }
    }

    fn marker_to_grid(width: u16, height: u16, marker: Marker) -> Box<dyn Grid> {
        match marker {
            Marker::Dot => Box::new(CharGrid::new(width, height, '•')),
            Marker::Block => Box::new(CharGrid::new(width, height, '█').apply_color_to_bg()),
            Marker::Bar => Box::new(CharGrid::new(width, height, '▄')),
            Marker::Braille => Box::new(PatternGrid::<2, 4>::new(width, height, &BRAILLE)),
            Marker::HalfBlock => Box::new(HalfBlockGrid::new(width, height)),
            Marker::Quadrant => Box::new(PatternGrid::<2, 2>::new(width, height, &QUADRANTS)),
            Marker::Sextant => Box::new(PatternGrid::<2, 3>::new(width, height, &SEXTANTS)),
            Marker::Octant => Box::new(PatternGrid::<2, 4>::new(width, height, &OCTANTS)),
            _ => Box::new(PatternGrid::<2, 4>::new(width, height, &BRAILLE)), // Fall back to braille if not supported.
        }
    }

    /// Draw any object that may implement the Shape trait
    pub fn draw<S>(&mut self, shape: &S)
    where
        S: Shape,
    {
        self.dirty = true;
        let mut painter = Painter::from(self);
        shape.draw(&mut painter);
    }
}

/// The Canvas widget may be used to draw more detailed figures using braille
/// patterns (each cell can have a braille character in 8 different positions).
pub struct Canvas<'a, F>
where
    F: Fn(&mut Context<'_>),
{
    block: Option<Block<'a>>,
    x_bounds: [f64; 2],
    y_bounds: [f64; 2],
    paint_func: Option<F>,
    background_color: Color,
    marker: Marker,
}

impl<'a, F> Default for Canvas<'a, F>
where
    F: Fn(&mut Context<'_>),
{
    fn default() -> Canvas<'a, F> {
        Canvas {
            block: None,
            x_bounds: [0.0, 0.0],
            y_bounds: [0.0, 0.0],
            paint_func: None,
            background_color: Color::Reset,
            marker: Marker::Braille,
        }
    }
}

impl<'a, F> Canvas<'a, F>
where
    F: Fn(&mut Context<'_>),
{
    pub fn x_bounds(mut self, bounds: [f64; 2]) -> Canvas<'a, F> {
        self.x_bounds = bounds;
        self
    }

    pub fn y_bounds(mut self, bounds: [f64; 2]) -> Canvas<'a, F> {
        self.y_bounds = bounds;
        self
    }

    /// Store the closure that will be used to draw to the Canvas
    pub fn paint(mut self, f: F) -> Canvas<'a, F> {
        self.paint_func = Some(f);
        self
    }

    pub fn background_color(mut self, color: Color) -> Canvas<'a, F> {
        self.background_color = color;
        self
    }

    /// Change the type of points used to draw the shapes. By default, the
    /// braille patterns are used as they provide a more fine-grained result,
    /// but you might want to use the simple dot or block instead if the
    /// targeted terminal does not support those symbols.
    pub fn marker(mut self, marker: Marker) -> Canvas<'a, F> {
        self.marker = marker;
        self
    }
}

impl<F> Widget for Canvas<'_, F>
where
    F: Fn(&mut Context<'_>),
{
    fn render(self, area: Rect, buf: &mut Buffer) {
        Widget::render(&self, area, buf);
    }
}

impl<F> Widget for &Canvas<'_, F>
where
    F: Fn(&mut Context<'_>),
{
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.block.as_ref().render(area, buf);
        let canvas_area = self.block.inner_if_some(area);
        if canvas_area.is_empty() {
            return;
        }

        buf.set_style(canvas_area, Style::default().bg(self.background_color));

        let width = canvas_area.width as usize;

        let Some(ref painter) = self.paint_func else {
            return;
        };

        // Create a blank context that match the size of the canvas
        let mut ctx = Context::new(
            canvas_area.width,
            canvas_area.height,
            self.x_bounds,
            self.y_bounds,
            self.marker,
        );
        // Paint to this context
        painter(&mut ctx);
        // ctx.finish(); // Not needed, we have no layers.

        // Instead, paint whatever is in the ctx.
        let layer = ctx.grid.save();

        for (index, layer_cell) in layer.contents.iter().enumerate() {
            let (x, y) = (
                (index % width) as u16 + canvas_area.left(),
                (index / width) as u16 + canvas_area.top(),
            );

            if let Some(cell) = buf.cell_mut((x, y)) {
                if let Some(symbol) = layer_cell.symbol {
                    cell.set_char(symbol);
                }
                if let Some(fg) = layer_cell.fg {
                    cell.set_fg(fg);
                }
                if let Some(bg) = layer_cell.bg {
                    cell.set_bg(bg);
                }
            }
        }

        // Reset the grid and mark as non-dirty.
        ctx.grid.reset();
        ctx.dirty = false;

        // Finally draw the labels
        let left = self.x_bounds[0];
        let right = self.x_bounds[1];
        let top = self.y_bounds[1];
        let bottom = self.y_bounds[0];
        let width = (self.x_bounds[1] - self.x_bounds[0]).abs();
        let height = (self.y_bounds[1] - self.y_bounds[0]).abs();
        let resolution = {
            let width = f64::from(canvas_area.width - 1);
            let height = f64::from(canvas_area.height - 1);
            (width, height)
        };
        for label in ctx
            .labels
            .iter()
            .filter(|l| l.x >= left && l.x <= right && l.y <= top && l.y >= bottom)
        {
            let x = ((label.x - left) * resolution.0 / width) as u16 + canvas_area.left();
            let y = ((top - label.y) * resolution.1 / height) as u16 + canvas_area.top();
            buf.set_line(x, y, &label.spans, canvas_area.right() - x);
        }
    }
}
