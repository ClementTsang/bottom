use std::{fmt::Debug, iter::zip};

use itertools::Itertools;
use tui::{style::Color, symbols};

#[derive(Debug, Clone)]
pub(super) struct Layer {
    pub(super) string: String,
    pub(super) colors: Vec<(Color, Color)>,
}

/// A [`Grid`] is a trait that represents a grid of cells, drawn in a
/// specific way.
pub(super) trait Grid: Debug {
    /// Get the resolution of the grid in number of dots.
    ///
    /// This doesn't have to be the same as the number of rows and columns of the grid. For example,
    /// a grid of Braille patterns will have a resolution of 2x4 dots per cell. This means that a
    /// grid of 10x10 cells will have a resolution of 20x40 dots.
    fn resolution(&self) -> (f64, f64);
    /// Paint a point of the grid.
    ///
    /// The point is expressed in number of dots starting at the origin of the grid in the top left
    /// corner. Note that this is not the same as the `(x, y)` coordinates of the canvas.
    fn paint(&mut self, x: usize, y: usize, color: Color);
    /// Save the current state of the [`Grid`] as a layer to be rendered
    fn save(&self) -> Layer;
    /// Reset the grid to its initial state
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
pub(super) struct BrailleGrid {
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
    pub(super) fn new(width: u16, height: u16) -> Self {
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
        // Note the braille array corresponds to:
        // ⠁⠈
        // ⠂⠐
        // ⠄⠠
        // ⡀⢀

        let index = y / 4 * self.width as usize + x / 2;

        // The ratatui/tui-rs implementation; this gives a more merged
        // look but it also makes it a bit harder to read in some cases.

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
                    *cell = symbols::braille::BLANK | symbols::braille::DOTS[y % 4][x % 2];
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
pub(super) struct CharGrid {
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
    pub(super) fn new(width: u16, height: u16, cell_char: char) -> Self {
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
pub(super) struct HalfBlockGrid {
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
    pub(super) fn new(width: u16, height: u16) -> Self {
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
