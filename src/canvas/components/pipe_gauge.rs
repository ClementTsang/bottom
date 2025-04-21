use tui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::Line,
    widgets::{Block, Widget},
};

#[derive(Debug, Clone, Copy)]
pub enum LabelLimit {
    None,
    #[expect(dead_code)]
    Auto(u16),
    Bars,
    StartLabel,
}

impl Default for LabelLimit {
    fn default() -> Self {
        Self::None
    }
}

/// A widget to measure something, using pipe characters ('|') as a unit.
#[derive(Debug, Clone)]
pub struct PipeGauge<'a> {
    block: Option<Block<'a>>,
    ratio: f64,
    start_label: Option<Line<'a>>,
    inner_label: Option<Line<'a>>,
    label_style: Style,
    gauge_style: Style,
    hide_parts: LabelLimit,
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

                let pipe_end =
                    start + (f64::from(end.saturating_sub(start)) * self.ratio).floor() as u16;
                for col in start..pipe_end {
                    if let Some(cell) = buf.cell_mut((col, row)) {
                        cell.set_symbol("|").set_style(Style {
                            fg: self.gauge_style.fg,
                            bg: None,
                            add_modifier: self.gauge_style.add_modifier,
                            sub_modifier: self.gauge_style.sub_modifier,
                            underline_color: None,
                        });
                    }
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
