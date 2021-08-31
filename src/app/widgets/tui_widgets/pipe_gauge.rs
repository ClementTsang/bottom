use tui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::Spans,
    widgets::{Block, Widget},
};

/// A widget to measure something, using pipe characters ('|') as a unit.
#[derive(Debug, Clone)]
pub struct PipeGauge<'a> {
    block: Option<Block<'a>>,
    ratio: f64,
    start_label: Option<Spans<'a>>,
    end_label: Option<Spans<'a>>,
    style: Style,
    gauge_style: Style,
}

impl<'a> Default for PipeGauge<'a> {
    fn default() -> Self {
        Self {
            block: None,
            ratio: 0.0,
            start_label: None,
            end_label: None,
            style: Style::default(),
            gauge_style: Style::default(),
        }
    }
}

impl<'a> PipeGauge<'a> {
    pub fn ratio(mut self, ratio: f64) -> Self {
        if ratio < 0.0 {
            self.ratio = 0.0;
        } else if ratio > 1.0 {
            self.ratio = 1.0;
        } else {
            self.ratio = ratio;
        }

        self
    }

    pub fn start_label<T>(mut self, start_label: T) -> Self
    where
        T: Into<Spans<'a>>,
    {
        self.start_label = Some(start_label.into());
        self
    }

    pub fn end_label<T>(mut self, end_label: T) -> Self
    where
        T: Into<Spans<'a>>,
    {
        self.end_label = Some(end_label.into());
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn gauge_style(mut self, style: Style) -> Self {
        self.gauge_style = style;
        self
    }
}

impl<'a> Widget for PipeGauge<'a> {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        buf.set_style(area, self.style);
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

        let ratio = self.ratio;
        let start_label = self
            .start_label
            .unwrap_or_else(move || Spans::from(format!("{:.0}%", ratio * 100.0)));

        let (col, row) = buf.set_spans(
            gauge_area.left(),
            gauge_area.top(),
            &start_label,
            gauge_area.width,
        );
        let (col, row) = buf.set_spans(col, row, &Spans::from("["), gauge_area.width);
        let start = col;
        if start >= gauge_area.right() {
            return;
        }

        let end_label = self.end_label.unwrap_or_default();
        let gauge_end: u16 =
            if (end_label.width() as u16) < (gauge_area.right().saturating_sub(start + 1)) {
                let gauge_end = gauge_area
                    .right()
                    .saturating_sub(end_label.width() as u16 + 1);
                {
                    let (col, row) = buf.set_spans(gauge_end, row, &end_label, gauge_area.width);
                    let _ = buf.set_spans(col, row, &Spans::from("]"), gauge_area.width);
                }

                gauge_end
            } else {
                let gauge_end = gauge_area.right().saturating_sub(1);

                let _ = buf.set_spans(gauge_end, row, &Spans::from("]"), gauge_area.width);
                gauge_end
            };

        let end = start + (f64::from(gauge_end.saturating_sub(start)) * self.ratio).floor() as u16;
        for col in start..end {
            buf.get_mut(col, row).set_symbol("|").set_style(Style {
                fg: self.gauge_style.fg,
                bg: None,
                add_modifier: self.gauge_style.add_modifier,
                sub_modifier: self.gauge_style.sub_modifier,
            });
        }
        for col in end..gauge_end {
            buf.get_mut(col, row).set_symbol(" ");
        }
    }
}
