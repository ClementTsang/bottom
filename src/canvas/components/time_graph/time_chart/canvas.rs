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

use std::{fmt::Debug, iter::zip};

use itertools::Itertools;
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

#[derive(Debug, Clone)]
struct Layer {
    string: String,
    colors: Vec<(Color, Color)>,
}

trait Grid: Debug {
    // fn width(&self) -> u16;
    // fn height(&self) -> u16;
    fn resolution(&self) -> (f64, f64);
    fn paint(&mut self, x: usize, y: usize, color: Color);
    fn save(&self) -> Layer;
    fn reset(&mut self);
}

/// The `BrailleGrid` is a grid made up of cells each containing a Braille pattern.
///
/// This makes it possible to draw shapes with a resolution of 2x4 dots per cell. This is useful
/// when you want to draw shapes with a high resolution. Font support for Braille patterns is
/// required to see the dots. If your terminal or font does not support this unicode block, you
/// will see unicode replacement characters (�) instead of braille dots.
///
/// This grid type only supports a single foreground color for each 2x4 dots cell. There is no way
/// to set the individual color of each dot in the braille pattern.
#[derive(Debug)]
struct BrailleGrid {
    /// Width of the grid in number of terminal columns
    width: u16,
    /// Height of the grid in number of terminal rows
    height: u16,
    /// Represents the unicode braille patterns. Will take a value between `0x2800` and `0x28FF`;
    /// this is converted to an utf16 string when converting to a layer. See
    /// <https://en.wikipedia.org/wiki/Braille_Patterns> for more info.
    ///
    /// FIXME: (points_rework_v1) isn't this really inefficient to go u16 -> String from utf16?
    utf16_code_points: Vec<u16>,
    /// The color of each cell only supports foreground colors for now as there's no way to
    /// individually set the background color of each dot in the braille pattern.
    colors: Vec<Color>,
}

impl BrailleGrid {
    /// Create a new `BrailleGrid` with the given width and height measured in terminal columns and
    /// rows respectively.
    fn new(width: u16, height: u16) -> Self {
        let length = usize::from(width * height);
        Self {
            width,
            height,
            utf16_code_points: vec![symbols::braille::BLANK; length],
            colors: vec![Color::Reset; length],
        }
    }
}

impl Grid for BrailleGrid {
    fn resolution(&self) -> (f64, f64) {
        (f64::from(self.width) * 2.0, f64::from(self.height) * 4.0)
    }

    fn save(&self) -> Layer {
        let string = String::from_utf16(&self.utf16_code_points).unwrap();
        // the background color is always reset for braille patterns
        let colors = self.colors.iter().map(|c| (*c, Color::Reset)).collect();
        Layer { string, colors }
    }

    fn reset(&mut self) {
        self.utf16_code_points.fill(symbols::braille::BLANK);
        self.colors.fill(Color::Reset);
    }

    fn paint(&mut self, x: usize, y: usize, color: Color) {
        let index = y / 4 * self.width as usize + x / 2;

        // if let Some(c) = self.utf16_code_points.get_mut(index) {
        //     *c |= symbols::braille::DOTS[y % 4][x % 2];
        // }
        // if let Some(c) = self.colors.get_mut(index) {
        //     *c = color;
        // }

        // Custom implementation to distinguish between lines better.
        if let Some(curr_color) = self.colors.get_mut(index) {
            if *curr_color != color {
                *curr_color = color;
                if let Some(cell) = self.utf16_code_points.get_mut(index) {
                    *cell = symbols::braille::BLANK;
                    *cell |= symbols::braille::DOTS[y % 4][x % 2];
                }
            } else if let Some(cell) = self.utf16_code_points.get_mut(index) {
                *cell |= symbols::braille::DOTS[y % 4][x % 2];
            }
        }
    }
}

/// The `CharGrid` is a grid made up of cells each containing a single character.
///
/// This makes it possible to draw shapes with a resolution of 1x1 dots per cell. This is useful
/// when you want to draw shapes with a low resolution.
#[derive(Debug)]
struct CharGrid {
    /// Width of the grid in number of terminal columns
    width: u16,
    /// Height of the grid in number of terminal rows
    height: u16,
    /// Represents a single character for each cell
    cells: Vec<char>,
    /// The color of each cell
    colors: Vec<Color>,
    /// The character to use for every cell - e.g. a block, dot, etc.
    cell_char: char,
}

impl CharGrid {
    /// Create a new `CharGrid` with the given width and height measured in terminal columns and
    /// rows respectively.
    fn new(width: u16, height: u16, cell_char: char) -> Self {
        let length = usize::from(width * height);
        Self {
            width,
            height,
            cells: vec![' '; length],
            colors: vec![Color::Reset; length],
            cell_char,
        }
    }
}

impl Grid for CharGrid {
    fn resolution(&self) -> (f64, f64) {
        (f64::from(self.width), f64::from(self.height))
    }

    fn save(&self) -> Layer {
        Layer {
            string: self.cells.iter().collect(),
            colors: self.colors.iter().map(|c| (*c, Color::Reset)).collect(),
        }
    }

    fn reset(&mut self) {
        self.cells.fill(' ');
        self.colors.fill(Color::Reset);
    }

    fn paint(&mut self, x: usize, y: usize, color: Color) {
        let index = y * self.width as usize + x;
        // using get_mut here because we are indexing the vector with usize values
        // and we want to make sure we don't panic if the index is out of bounds
        if let Some(c) = self.cells.get_mut(index) {
            *c = self.cell_char;
        }
        if let Some(c) = self.colors.get_mut(index) {
            *c = color;
        }
    }
}

/// The `HalfBlockGrid` is a grid made up of cells each containing a half block character.
///
/// In terminals, each character is usually twice as tall as it is wide. Unicode has a couple of
/// vertical half block characters, the upper half block '▀' and lower half block '▄' which take up
/// half the height of a normal character but the full width. Together with an empty space ' ' and a
/// full block '█', we can effectively double the resolution of a single cell. In addition, because
/// each character can have a foreground and background color, we can control the color of the upper
/// and lower half of each cell. This allows us to draw shapes with a resolution of 1x2 "pixels" per
/// cell.
///
/// This allows for more flexibility than the `BrailleGrid` which only supports a single
/// foreground color for each 2x4 dots cell, and the `CharGrid` which only supports a single
/// character for each cell.
#[derive(Debug)]
struct HalfBlockGrid {
    /// Width of the grid in number of terminal columns
    width: u16,
    /// Height of the grid in number of terminal rows
    height: u16,
    /// Represents a single color for each "pixel" arranged in column, row order
    pixels: Vec<Vec<Color>>,
}

impl HalfBlockGrid {
    /// Create a new `HalfBlockGrid` with the given width and height measured in terminal columns
    /// and rows respectively.
    fn new(width: u16, height: u16) -> Self {
        Self {
            width,
            height,
            pixels: vec![vec![Color::Reset; width as usize]; height as usize * 2],
        }
    }
}

impl Grid for HalfBlockGrid {
    fn resolution(&self) -> (f64, f64) {
        (f64::from(self.width), f64::from(self.height) * 2.0)
    }

    fn save(&self) -> Layer {
        // Given that we store the pixels in a grid, and that we want to use 2 pixels arranged
        // vertically to form a single terminal cell, which can be either empty, upper half block,
        // lower half block or full block, we need examine the pixels in vertical pairs to decide
        // what character to print in each cell. So these are the 4 states we use to represent each
        // cell:
        //
        // 1. upper: reset, lower: reset => ' ' fg: reset / bg: reset
        // 2. upper: reset, lower: color => '▄' fg: lower color / bg: reset
        // 3. upper: color, lower: reset => '▀' fg: upper color / bg: reset
        // 4. upper: color, lower: color => '▀' fg: upper color / bg: lower color
        //
        // Note that because the foreground reset color (i.e. default foreground color) is usually
        // not the same as the background reset color (i.e. default background color), we need to
        // swap around the colors for that state (2 reset/color).
        //
        // When the upper and lower colors are the same, we could continue to use an upper half
        // block, but we choose to use a full block instead. This allows us to write unit tests that
        // treat the cell as a single character instead of two half block characters.

        // first we join each adjacent row together to get an iterator that contains vertical pairs
        // of pixels, with the lower row being the first element in the pair
        //
        // TODO: Whenever I add this as a valid marker, make sure this works fine with
        // the overridden time_chart drawing-layer-thing.
        let vertical_color_pairs = self
            .pixels
            .iter()
            .tuples()
            .flat_map(|(upper_row, lower_row)| zip(upper_row, lower_row));

        // then we work out what character to print for each pair of pixels
        let string = vertical_color_pairs
            .clone()
            .map(|(upper, lower)| match (upper, lower) {
                (Color::Reset, Color::Reset) => ' ',
                (Color::Reset, _) => symbols::half_block::LOWER,
                (_, Color::Reset) => symbols::half_block::UPPER,
                (&lower, &upper) => {
                    if lower == upper {
                        symbols::half_block::FULL
                    } else {
                        symbols::half_block::UPPER
                    }
                }
            })
            .collect();

        // then we convert these each vertical pair of pixels into a foreground and background color
        let colors = vertical_color_pairs
            .map(|(upper, lower)| {
                let (fg, bg) = match (upper, lower) {
                    (Color::Reset, Color::Reset) => (Color::Reset, Color::Reset),
                    (Color::Reset, &lower) => (lower, Color::Reset),
                    (&upper, Color::Reset) => (upper, Color::Reset),
                    (&upper, &lower) => (upper, lower),
                };
                (fg, bg)
            })
            .collect();

        Layer { string, colors }
    }

    fn reset(&mut self) {
        self.pixels.fill(vec![Color::Reset; self.width as usize]);
    }

    fn paint(&mut self, x: usize, y: usize, color: Color) {
        self.pixels[y][x] = color;
    }
}

#[derive(Debug)]
pub struct Painter<'a, 'b> {
    context: &'a mut Context<'b>,
    resolution: (f64, f64),
}

impl Painter<'_, '_> {
    /// Convert the (x, y) coordinates to location of a point on the grid.
    pub fn get_point(&self, x: f64, y: f64) -> Option<(usize, usize)> {
        let left = self.context.x_bounds[0];
        let right = self.context.x_bounds[1];
        let top = self.context.y_bounds[1];
        let bottom = self.context.y_bounds[0];
        if x < left || x > right || y < bottom || y > top {
            return None;
        }
        let width = (self.context.x_bounds[1] - self.context.x_bounds[0]).abs();
        let height = (self.context.y_bounds[1] - self.context.y_bounds[0]).abs();
        if width == 0.0 || height == 0.0 {
            return None;
        }
        let x = ((x - left) * self.resolution.0 / width) as usize;
        let y = ((top - y) * self.resolution.1 / height) as usize;
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
            if ch != ' ' && ch != '\u{2800}' {
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
