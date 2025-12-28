//! Vendored [starting from here](https://github.com/ratatui/ratatui/blob/65c520245aa20e99e64d9ffcb2062a4502a699ea/ratatui-widgets/src/canvas.rs#L322).

use std::{fmt::Debug, iter::zip};

use itertools::Itertools;
use tui::{style::Color, symbols};

/// A single layer of the canvas.
///
/// This allows the canvas to be drawn in multiple layers. This is useful if you want to draw
/// multiple shapes on the canvas in specific order.
///
/// **NOTE**: In the vendored version, we don't ever actually want to do this.
#[derive(Debug)]
pub(super) struct Layer {
    pub(super) contents: Vec<LayerCell>,
}

/// A cell within a layer.
///
/// If a [`Context`] contains multiple layers, then the symbol, foreground, and background colors
/// for a character will be determined by the top-most layer that provides a value for that
/// character. For example, a chart drawn with [`Marker::Block`] may provide the background color,
/// and a later chart drawn with [`Marker::Braille`] may provide the symbol and foreground color.
#[derive(Debug)]
pub(super) struct LayerCell {
    pub(super) symbol: Option<char>,
    pub(super) fg: Option<Color>,
    pub(super) bg: Option<Color>,
}

/// A grid of cells that can be painted on.
///
/// The grid represents a particular screen region measured in rows and columns. The underlying
/// resolution of the grid might exceed the number of rows and columns. For example, a grid of
/// Braille patterns will have a resolution of 2x4 dots per cell. This means that a grid of 10x10
/// cells will have a resolution of 20x40 dots.
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

/// The pattern and colour of a `PatternGrid` cell.
#[derive(Copy, Clone, Debug, Default)]
struct PatternCell {
    /// The pattern of a grid character.
    ///
    /// The pattern is stored in the lower bits in a row-major order. For instance, for a 2x4
    /// pattern marker, bits 0 to 7 of this field should represent the following pseudo-pixels:
    ///
    /// | 0 1 |
    /// | 2 3 |
    /// | 4 5 |
    /// | 6 7 |
    pattern: u8,
    /// The color of a cell only supports foreground colors for now as there's no way to
    /// individually set the background color of each pseudo-pixel in a pattern character.
    color: Option<Color>,
}

/// The `PatternGrid` is a grid made up of cells each containing a `W`x`H` pattern character.
///
/// This makes it possible to draw shapes with a resolution of e.g. 2x4 (Braille or Unicode octant)
/// per cell.
/// Font support for the relevant pattern character is required. If your terminal or font does not
/// support the relevant Unicode block, you will see Unicode replacement characters (�) instead.
///
/// This grid type only supports a single foreground colour for each `W`x`H` pattern character.
/// There is no way to set the individual colour of each pseudo-pixel.
#[derive(Debug)]
pub(super) struct PatternGrid<const W: usize, const H: usize> {
    /// Width of the grid in number of terminal columns
    width: u16,
    /// Height of the grid in number of terminal rows
    height: u16,
    /// Pattern and color of the cells.
    cells: Vec<PatternCell>,
    /// Lookup table mapping patterns to characters.
    char_table: &'static [char],
}

impl<const W: usize, const H: usize> PatternGrid<W, H> {
    /// Statically check that the dimension of the pattern is supported.
    const _PATTERN_DIMENSION_CHECK: usize = u8::BITS as usize - W * H;

    /// Create a new `PatternGrid` with the given width and height measured in terminal columns
    /// and rows respectively.
    pub(super) fn new(width: u16, height: u16, char_table: &'static [char]) -> Self {
        // Cause a static error if the pattern doesn't fit within 8 bits.
        let _ = Self::_PATTERN_DIMENSION_CHECK;

        let length = usize::from(width) * usize::from(height);
        Self {
            width,
            height,
            cells: vec![PatternCell::default(); length],
            char_table,
        }
    }
}

impl<const W: usize, const H: usize> Grid for PatternGrid<W, H> {
    fn resolution(&self) -> (f64, f64) {
        (
            f64::from(self.width) * W as f64,
            f64::from(self.height) * H as f64,
        )
    }

    fn paint(&mut self, x: usize, y: usize, color: Color) {
        let index = y
            .saturating_div(H)
            .saturating_mul(self.width as usize)
            .saturating_add(x.saturating_div(W));

        // The ratatui/tui-rs implementation; this gives a more merged
        // look, but it also makes it a bit harder to read in some cases.
        //
        // using get_mut here because we are indexing the vector with usize values
        // and we want to make sure we don't panic if the index is out of bounds
        // if let Some(cell) = self.cells.get_mut(index) {
        //     cell.pattern |= 1u8 << ((x % W) + W * (y % H));
        //     cell.color = Some(color);
        // }

        // Custom implementation do distinguish between lines better.
        if let Some(cell) = self.cells.get_mut(index) {
            if let Some(curr_color) = &mut cell.color {
                if *curr_color != color {
                    *curr_color = color;
                    cell.pattern = 1u8 << ((x % W) + W * (y % H));
                } else {
                    cell.pattern |= 1u8 << ((x % W) + W * (y % H));
                }
            } else {
                cell.pattern |= 1u8 << ((x % W) + W * (y % H));
            }
        }
    }

    fn save(&self) -> Layer {
        let contents = self
            .cells
            .iter()
            .map(|&cell| {
                let symbol = match cell.pattern {
                    // Skip rendering blank patterns to allow layers underneath
                    // to show through.
                    0 => None,
                    idx => Some(self.char_table[idx as usize]),
                };

                LayerCell {
                    symbol,
                    fg: cell.color,
                    // Patterns only affect foreground.
                    bg: None,
                }
            })
            .collect();

        Layer { contents }
    }

    fn reset(&mut self) {
        self.cells.fill_with(Default::default);
    }
}

// impl Grid for BrailleGrid {
//     fn resolution(&self) -> (f64, f64) {
//         (f64::from(self.width) * 2.0, f64::from(self.height) * 4.0)
//     }
//
//     fn paint(&mut self, x: usize, y: usize, color: Color) {
//         // Note the braille array corresponds to:
//         // ```
//         // ⠁⠈
//         // ⠂⠐
//         // ⠄⠠
//         // ⡀⢀
//         // ```
//         const BLANK: u16 = 0x2800;
//         const DOTS: [[u16; 2]; 4] = [
//             [0x0001, 0x0008],
//             [0x0002, 0x0010],
//             [0x0004, 0x0020],
//             [0x0040, 0x0080],
//         ];
//
//         let index = y / 4 * self.width as usize + x / 2;
//
//         // The ratatui/tui-rs implementation; this gives a more merged
//         // look, but it also makes it a bit harder to read in some cases.
//
//         // if let Some(c) = self.utf16_code_points.get_mut(index) {
//         //     *c |= DOTS[y % 4][x % 2];
//         // }
//         // if let Some(c) = self.colors.get_mut(index) {
//         //     *c = color;
//         // }
//
//         // Custom implementation to distinguish between lines better.
//         if let Some(curr_color) = self.colors.get_mut(index) {
//             if *curr_color != color {
//                 *curr_color = color;
//                 if let Some(cell) = self.utf16_code_points.get_mut(index) {
//                     *cell = BLANK | DOTS[y % 4][x % 2];
//                 }
//             } else if let Some(cell) = self.utf16_code_points.get_mut(index) {
//                 *cell |= DOTS[y % 4][x % 2];
//             }
//         }
//     }
//
//     fn save(&self) -> Layer {
//         let string = String::from_utf16(&self.utf16_code_points).expect("valid UTF-16 data");
//         // the background color is always reset for braille patterns
//         let colors = self.colors.iter().map(|c| (*c, Color::Reset)).collect();
//         Layer { string, colors }
//     }
//
//     fn reset(&mut self) {
//         self.utf16_code_points.fill(BLANK);
//         self.colors.fill(Color::Reset);
//     }
// }

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
    /// The color of each cell
    cells: Vec<Option<Color>>,

    /// The character to use for every cell - e.g. a block, dot, etc.
    cell_char: char,

    /// If true, apply the color to the background as well as the foreground. This is used for
    /// [`Marker::Block`], so that it will overwrite any previous foreground character, but also
    /// leave a background that can be overlaid with an additional foreground character.
    apply_color_to_bg: bool,
}

impl CharGrid {
    /// Create a new `CharGrid` with the given width and height measured in terminal columns and
    /// rows respectively.
    pub(super) fn new(width: u16, height: u16, cell_char: char) -> Self {
        let length = usize::from(width) * usize::from(height);
        Self {
            width,
            height,
            cells: vec![None; length],
            cell_char,
            apply_color_to_bg: false,
        }
    }

    pub(super) fn apply_color_to_bg(self) -> Self {
        Self {
            apply_color_to_bg: true,
            ..self
        }
    }
}

impl Grid for CharGrid {
    fn resolution(&self) -> (f64, f64) {
        (f64::from(self.width), f64::from(self.height))
    }

    fn paint(&mut self, x: usize, y: usize, color: Color) {
        let index = y.saturating_mul(self.width as usize).saturating_add(x);
        // using get_mut here because we are indexing the vector with usize values
        // and we want to make sure we don't panic if the index is out of bounds
        if let Some(c) = self.cells.get_mut(index) {
            *c = Some(color);
        }
    }

    fn save(&self) -> Layer {
        Layer {
            contents: self
                .cells
                .iter()
                .map(|&color| LayerCell {
                    symbol: color.map(|_| self.cell_char),
                    fg: color,
                    bg: color.filter(|_| self.apply_color_to_bg),
                })
                .collect(),
        }
    }

    fn reset(&mut self) {
        self.cells.fill(None);
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
/// This allows for more flexibility than the `PatternGrid` which only supports a single
/// foreground color for each 2x4 dots cell, and the `CharGrid` which only supports a single
/// character for each cell.
#[derive(Debug)]

pub(super) struct HalfBlockGrid {
    /// Width of the grid in number of terminal columns
    width: u16,
    /// Height of the grid in number of terminal rows
    height: u16,
    /// Represents a single color for each "pixel" arranged in column, row order
    pixels: Vec<Vec<Option<Color>>>,
}

impl HalfBlockGrid {
    /// Create a new `HalfBlockGrid` with the given width and height measured in terminal columns
    /// and rows respectively.
    pub(super) fn new(width: u16, height: u16) -> Self {
        Self {
            width,
            height,
            pixels: vec![vec![None; width as usize]; (height as usize) * 2],
        }
    }
}

impl Grid for HalfBlockGrid {
    fn resolution(&self) -> (f64, f64) {
        (f64::from(self.width), f64::from(self.height) * 2.0)
    }

    fn paint(&mut self, x: usize, y: usize, color: Color) {
        self.pixels[y][x] = Some(color);
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
        let vertical_color_pairs = self
            .pixels
            .iter()
            .tuples()
            .flat_map(|(upper_row, lower_row)| zip(upper_row, lower_row));

        // Then we determine the character to print for each pair, along with the color of the
        // foreground and background.
        let contents = vertical_color_pairs
            .map(|(upper, lower)| {
                let (symbol, fg, bg) = match (upper, lower) {
                    (None, None) => (None, None, None),
                    (None, Some(lower)) => (Some(symbols::half_block::LOWER), Some(*lower), None),
                    (Some(upper), None) => (Some(symbols::half_block::UPPER), Some(*upper), None),
                    (Some(upper), Some(lower)) if lower == upper => {
                        (Some(symbols::half_block::FULL), Some(*upper), Some(*lower))
                    }
                    (Some(upper), Some(lower)) => {
                        (Some(symbols::half_block::UPPER), Some(*upper), Some(*lower))
                    }
                };
                LayerCell { symbol, fg, bg }
            })
            .collect();

        Layer { contents }
    }

    fn reset(&mut self) {
        self.pixels.fill(vec![None; self.width as usize]);
    }
}
