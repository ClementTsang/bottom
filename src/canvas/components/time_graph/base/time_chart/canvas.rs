//! Vendored from <https://github.com/fdehau/tui-rs/blob/fafad6c96109610825aad89c4bba5253e01101ed/src/widgets/canvas/mod.rs>
//! and <https://github.com/ratatui-org/ratatui/blob/c8dd87918d44fff6d4c3c78e1fc821a3275db1ae/src/widgets/canvas.rs>.
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

use tui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    symbols,
    text::Line,
    widgets::{
        Block, Widget,
        canvas::{Line as CanvasLine, Points},
    },
};

use super::grid::{BrailleGrid, CharGrid, Grid, HalfBlockGrid};

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

#[derive(Debug, Clone)]
pub struct FilledLine {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
    pub color: Color,
    pub baseline: Option<f64>,
}

impl Shape for FilledLine {
    fn draw(&self, painter: &mut Painter<'_, '_>) {
        let (x1, y1) = match painter.get_point(self.x1, self.y1) {
            Some(c) => c,
            None => return,
        };
        let (x2, y2) = match painter.get_point(self.x2, self.y2) {
            Some(c) => c,
            None => return,
        };
        let (x1, y1, x2, y2) = if x1 > x2 {
            (x2, y2, x1, y1)
        } else {
            (x1, y1, x2, y2)
        };

        let dx = (x2 as isize - x1 as isize).abs();
        let dy = (y2 as isize - y1 as isize).abs();

        if dx >= dy {
            let mut d = 2 * dy - dx;
            let mut y = y1;
            for x in x1..=x2 {
                if let Some(baseline) = self.baseline {
                    painter.paint_range_floats(x, y as f64, baseline, self.color);
                } else {
                    painter.paint_column(x, y, self.color);
                }

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
        } else {
            let (x1, y1, x2, y2) = if y1 > y2 {
                (x2, y2, x1, y1)
            } else {
                (x1, y1, x2, y2)
            };
            let mut d = 2 * dx - dy;
            let mut x = x1;
            for y in y1..=y2 {
                if let Some(baseline) = self.baseline {
                    painter.paint_range_floats(x, y as f64, baseline, self.color);
                } else {
                    painter.paint_column(x, y, self.color);
                }

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

    /// Paint a column of the grid from y to the bottom.
    pub fn paint_column(&mut self, x: usize, y: usize, color: Color) {
        let max_y = self.resolution.1 as usize;
        for iy in y..max_y {
            self.context.grid.paint(x, iy, color);
        }
    }

    /// Paints a range of cells, mapping the float `baseline` to the grid.
    /// Handles out-of-bounds baselines by clamping to top/bottom edges.
    pub fn paint_range_floats(&mut self, x: usize, start_y_idx: f64, baseline: f64, color: Color) {
        let [bottom_val, top_val] = self.context.y_bounds;
        let height = top_val - bottom_val;
        let max_y_idx = self.resolution.1 as usize;

        // Calculate baseline Y index
        let baseline_y_idx = if height <= 0.0 {
            max_y_idx // Fallback
        } else {
            // Logic mirrors get_point but handles out-of-bounds slightly differently for filling
            // y index 0 is TOP. y index MAX is BOTTOM.
            // Formula: y_idx = ((top - y_val) * (res - 1) / height)

            let calc_idx = |val: f64| -> isize {
                ((top_val - val) * (self.resolution.1 - 1.0) / height).round() as isize
            };

            let idx = calc_idx(baseline);

            if idx < 0 {
                0 // Top
            } else if idx >= max_y_idx as isize {
                max_y_idx - 1 // Bottom
            } else {
                idx as usize
            }
        };

        let start = start_y_idx as usize;
        let end = baseline_y_idx;

        let (low, mut high) = if start < end {
            (start, end)
        } else {
            (end, start)
        };

        // Ensure within bounds (should exist but just in case)
        if low >= max_y_idx {
            return;
        }
        if high >= max_y_idx {
            high = max_y_idx - 1;
        }

        for iy in low..=high {
            self.context.grid.paint(x, iy, color);
        }
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
        width: u16, height: u16, x_bounds: [f64; 2], y_bounds: [f64; 2], marker: symbols::Marker,
    ) -> Context<'a> {
        let grid: Box<dyn Grid> = match marker {
            symbols::Marker::Dot => Box::new(CharGrid::new(width, height, '•')),
            symbols::Marker::Block => Box::new(CharGrid::new(width, height, '█')),
            symbols::Marker::Bar => Box::new(CharGrid::new(width, height, '▄')),
            symbols::Marker::Braille => Box::new(BrailleGrid::new(width, height)),
            symbols::Marker::HalfBlock => Box::new(HalfBlockGrid::new(width, height)),
        };
        Context {
            x_bounds,
            y_bounds,
            grid,
            dirty: false,
            labels: Vec::new(),
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
    painter: Option<F>,
    background_color: Color,
    marker: symbols::Marker,
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
            painter: None,
            background_color: Color::Reset,
            marker: symbols::Marker::Braille,
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
        self.painter = Some(f);
        self
    }

    pub fn background_color(mut self, color: Color) -> Canvas<'a, F> {
        self.background_color = color;
        self
    }

    /// Change the type of points used to draw the shapes. By default the
    /// braille patterns are used as they provide a more fine grained result
    /// but you might want to use the simple dot or block instead if the
    /// targeted terminal does not support those symbols.
    pub fn marker(mut self, marker: symbols::Marker) -> Canvas<'a, F> {
        self.marker = marker;
        self
    }
}

impl<F> Widget for Canvas<'_, F>
where
    F: Fn(&mut Context<'_>),
{
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        let canvas_area = match self.block.take() {
            Some(b) => {
                let inner_area = b.inner(area);
                b.render(area, buf);
                inner_area
            }
            None => area,
        };

        buf.set_style(canvas_area, Style::default().bg(self.background_color));

        let width = canvas_area.width as usize;

        let painter = match self.painter {
            Some(ref p) => p,
            None => return,
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

        // Paint whatever is in the ctx.
        let layer = ctx.grid.save();

        for (i, (ch, (fg, bg))) in layer
            .string
            .chars()
            .zip(layer.colors.into_iter())
            .enumerate()
        {
            const BRAILLE_BASE: char = '\u{2800}';
            if ch != ' ' && ch != BRAILLE_BASE {
                let (x, y) = (i % width, i / width);
                if let Some(cell) =
                    buf.cell_mut((x as u16 + canvas_area.left(), y as u16 + canvas_area.top()))
                {
                    cell.set_char(ch).set_fg(fg).set_bg(bg);
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
