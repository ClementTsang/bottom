use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    symbols,
    text::Line,
    widgets::{Block, Widget},
};

#[derive(Debug, Clone, Copy, Default)]
pub enum LabelLimit {
    #[default]
    None,
    #[expect(dead_code)]
    Auto(u16),
    Bars,
    StartLabel,
}

/// What bar character type to use.
#[derive(Debug, Clone, Default, Copy)]
pub enum BarType {
    #[default]
    /// The pipe character (`|`)
    Pipe,
    /// Bar characters (`█`, `▉`, etc.)
    Bar,
    /// Square characters (`■`)
    Square,
}

impl BarType {
    #[expect(dead_code)]
    fn is_pipe(&self) -> bool {
        matches!(self, BarType::Pipe)
    }

    fn is_bar(&self) -> bool {
        matches!(self, BarType::Bar)
    }
}

fn get_unicode_block<'a>(frac: f64) -> &'a str {
    match (frac * 8.0).round() as u16 {
        0 => " ",
        1 => symbols::block::ONE_EIGHTH,
        2 => symbols::block::ONE_QUARTER,
        3 => symbols::block::THREE_EIGHTHS,
        4 => symbols::block::HALF,
        5 => symbols::block::FIVE_EIGHTHS,
        6 => symbols::block::THREE_QUARTERS,
        7 => symbols::block::SEVEN_EIGHTHS,
        8 => symbols::block::FULL,
        _ => unreachable!("this case should never occur"),
    }
}

/// A widget to measure something, using pipe characters ('|') or horizontal bar characters as a unit.
#[derive(Debug, Clone)]
pub struct PipeGauge<'a> {
    block: Option<Block<'a>>,
    ratio: f64,
    start_label: Option<Line<'a>>,
    inner_label: Option<Line<'a>>,
    label_style: Style,
    gauge_style: Style,
    hide_parts: LabelLimit,
    bar_type: BarType,
}

impl Default for PipeGauge<'_> {
    fn default() -> Self {
        Self {
            block: None,
            ratio: 0.0,
            start_label: None,
            inner_label: None,
            label_style: Style::default(),
            gauge_style: Style::default(),
            hide_parts: LabelLimit::default(),
            bar_type: BarType::default(),
        }
    }
}

impl<'a> PipeGauge<'a> {
    /// The ratio, a value from 0.0 to 1.0 (any other greater or less will be
    /// clamped) represents the portion of the pipe gauge to fill.
    ///
    /// Note: passing in NaN will potentially cause problems.
    pub fn ratio(mut self, ratio: f64) -> Self {
        self.ratio = ratio.clamp(0.0, 1.0);

        self
    }

    /// The label displayed before the bar.
    pub fn start_label<T>(mut self, start_label: T) -> Self
    where
        T: Into<Line<'a>>,
    {
        self.start_label = Some(start_label.into());
        self
    }

    /// The label displayed inside the bar.
    pub fn inner_label<T>(mut self, inner_label: T) -> Self
    where
        T: Into<Line<'a>>,
    {
        self.inner_label = Some(inner_label.into());
        self
    }

    /// The style of the labels.
    pub fn label_style(mut self, label_style: Style) -> Self {
        self.label_style = label_style;
        self
    }

    /// The style of the gauge itself.
    pub fn gauge_style(mut self, style: Style) -> Self {
        self.gauge_style = style;
        self
    }

    /// Whether to hide parts of the gauge/label if the inner label wouldn't
    /// fit.
    pub fn hide_parts(mut self, hide_parts: LabelLimit) -> Self {
        self.hide_parts = hide_parts;
        self
    }

    /// What type of bar character to use.
    pub fn bar_type(mut self, bar_type: BarType) -> Self {
        self.bar_type = bar_type;
        self
    }
}

impl Widget for PipeGauge<'_> {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        buf.set_style(area, self.label_style);
        let gauge_area = match self.block.take() {
            Some(b) => {
                let inner_area = b.inner(area);
                b.render(area, buf);
                inner_area
            }
            None => area,
        };

        if gauge_area.height < 1 {
            return;
        }

        let (col, row) = {
            let inner_label_width = self
                .inner_label
                .as_ref()
                .map(|l| l.width())
                .unwrap_or_default();

            let start_label_width = self
                .start_label
                .as_ref()
                .map(|l| l.width())
                .unwrap_or_default();

            match self.hide_parts {
                LabelLimit::StartLabel => {
                    let inner_label = self.inner_label.unwrap_or_else(|| Line::from(""));
                    let _ = buf.set_line(
                        gauge_area.left(),
                        gauge_area.top(),
                        &inner_label,
                        inner_label.width() as u16,
                    );

                    // Short circuit.
                    return;
                }
                LabelLimit::Auto(_)
                    if gauge_area.width < (inner_label_width + start_label_width + 1) as u16 =>
                {
                    let inner_label = self.inner_label.unwrap_or_else(|| Line::from(""));
                    let _ = buf.set_line(
                        gauge_area.left(),
                        gauge_area.top(),
                        &inner_label,
                        inner_label.width() as u16,
                    );

                    // Short circuit.
                    return;
                }
                _ => {
                    let start_label = self.start_label.unwrap_or_else(|| Line::from(""));
                    buf.set_line(
                        gauge_area.left(),
                        gauge_area.top(),
                        &start_label,
                        start_label.width() as u16,
                    )
                }
            }
        };

        let end_label = self.inner_label.unwrap_or_else(|| Line::from(""));
        match self.hide_parts {
            LabelLimit::Bars => {
                let _ = buf.set_line(
                    gauge_area
                        .right()
                        .saturating_sub(end_label.width() as u16 + 1),
                    row,
                    &end_label,
                    end_label.width() as u16,
                );
            }
            LabelLimit::Auto(width_limit)
                if gauge_area.right().saturating_sub(col) < width_limit =>
            {
                let _ = buf.set_line(
                    gauge_area
                        .right()
                        .saturating_sub(end_label.width() as u16 + 1),
                    row,
                    &end_label,
                    1,
                );
            }
            LabelLimit::Auto(_) | LabelLimit::None => {
                // FIXME: "[" and "]" don't look that great with block bars.
                let (start, _) = buf.set_line(col, row, &Line::from("["), gauge_area.width);
                if start >= gauge_area.right() {
                    return;
                }
                let (end, _) = buf.set_line(
                    (gauge_area.x + gauge_area.width).saturating_sub(1),
                    row,
                    &Line::from("]"),
                    gauge_area.width,
                );

                let filled_width = f64::from(end.saturating_sub(start)) * self.ratio;
                let bar_end = end.saturating_sub(1);
                let pipe_end = bar_end.min(start + filled_width.floor() as u16);

                let symbol = match self.bar_type {
                    BarType::Pipe => "|",
                    BarType::Bar => symbols::block::FULL,
                    BarType::Square => "■",
                };

                let bar_style = Style {
                    fg: self.gauge_style.fg,
                    bg: None,
                    add_modifier: self.gauge_style.add_modifier,
                    sub_modifier: self.gauge_style.sub_modifier,
                    underline_color: None,
                };

                for col in start..pipe_end {
                    if let Some(cell) = buf.cell_mut((col, row)) {
                        cell.set_symbol(symbol).set_style(bar_style);
                    }
                }

                // Unlike pipes, blocks can also show the leftover fraction of a cell.
                // Based on what Ratatui does!
                if self.bar_type.is_bar()
                    && pipe_end < bar_end
                    && let Some(cell) = buf.cell_mut((pipe_end, row))
                {
                    cell.set_symbol(get_unicode_block(filled_width.fract()))
                        .set_style(bar_style);
                }

                if (end_label.width() as u16) < end.saturating_sub(start) {
                    let gauge_end = gauge_area
                        .right()
                        .saturating_sub(end_label.width() as u16 + 1);
                    buf.set_line(gauge_end, row, &end_label, end_label.width() as u16);
                }
            }
            LabelLimit::StartLabel => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_unicode_block() {
        // Each exact eighth should map to its own block.
        assert_eq!(get_unicode_block(0.0), " ");
        assert_eq!(get_unicode_block(0.125), symbols::block::ONE_EIGHTH);
        assert_eq!(get_unicode_block(0.25), symbols::block::ONE_QUARTER);
        assert_eq!(get_unicode_block(0.375), symbols::block::THREE_EIGHTHS);
        assert_eq!(get_unicode_block(0.5), symbols::block::HALF);
        assert_eq!(get_unicode_block(0.625), symbols::block::FIVE_EIGHTHS);
        assert_eq!(get_unicode_block(0.75), symbols::block::THREE_QUARTERS);
        assert_eq!(get_unicode_block(0.875), symbols::block::SEVEN_EIGHTHS);
        assert_eq!(get_unicode_block(1.0), symbols::block::FULL);

        // Anything in between should round to the nearest eighth.
        assert_eq!(get_unicode_block(0.05), " ");
        assert_eq!(get_unicode_block(0.1), symbols::block::ONE_EIGHTH);
        assert_eq!(get_unicode_block(0.3), symbols::block::ONE_QUARTER);
        assert_eq!(get_unicode_block(0.4), symbols::block::THREE_EIGHTHS);
        assert_eq!(get_unicode_block(0.55), symbols::block::HALF);
        assert_eq!(get_unicode_block(0.6), symbols::block::FIVE_EIGHTHS);
        assert_eq!(get_unicode_block(0.8), symbols::block::THREE_QUARTERS);
        assert_eq!(get_unicode_block(0.9), symbols::block::SEVEN_EIGHTHS);
        assert_eq!(get_unicode_block(0.99), symbols::block::FULL);
    }

    /// Draw a label-less gauge into a 12-wide area, which leaves 10 cells
    /// between the brackets.
    fn render_bar(ratio: f64, bar_type: BarType) -> String {
        const WIDTH: u16 = 12;

        let area = Rect::new(0, 0, WIDTH, 1);
        let mut buf = Buffer::empty(area);
        PipeGauge::default()
            .ratio(ratio)
            .bar_type(bar_type)
            .render(area, &mut buf);

        (0..WIDTH).map(|x| buf[(x, 0)].symbol()).collect()
    }

    #[test]
    fn test_pipe_bars() {
        assert_eq!(render_bar(0.0, BarType::Pipe), "[          ]");
        assert_eq!(render_bar(0.5, BarType::Pipe), "[|||||     ]");
        assert_eq!(render_bar(1.0, BarType::Pipe), "[||||||||||]");
    }

    #[test]
    fn test_solid_bars() {
        assert_eq!(render_bar(0.0, BarType::Bar), "[          ]");
        assert_eq!(render_bar(0.5, BarType::Bar), "[█████▌    ]");
        assert_eq!(render_bar(1.0, BarType::Bar), "[██████████]");
    }

    /// A partial block should never be drawn in place of a full one, or the
    /// bar would show less than it should.
    #[test]
    fn test_solid_bars_are_monotonic() {
        let mut prev = 0;

        for ratio in (0..=100).map(f64::from) {
            let bar = render_bar(ratio / 100.0, BarType::Bar);
            let filled = bar
                .chars()
                .filter(|c| *c != ' ' && *c != '[' && *c != ']')
                .count();

            assert!(
                filled >= prev,
                "bar at {ratio}% ({bar}) is shorter than the one before it"
            );
            prev = filled;
        }
    }
}
