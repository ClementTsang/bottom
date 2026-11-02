use std::{borrow::Cow, cmp::Ordering, num::NonZeroU16, time::Instant};

use concat_string::concat_string;
use ratatui::widgets::Row;

use crate::{
    app::AppConfigFields,
    canvas::{
        Painter,
        components::data_table::{
            Column, ColumnHeader, DataTable, DataTableColumn, DataTableProps, DataTableStyling,
            DataToCell,
        },
    },
    collection::cpu::{CpuData, CpuDataType},
    components::time_series::{PercentTimeGraph, TimeseriesConfig},
    options::config::{cpu::CpuDefault, style::Styles},
};

pub enum CpuWidgetColumn {
    Cpu,
    Use { show_decimal: bool },
}

impl ColumnHeader for CpuWidgetColumn {
    fn text(&self) -> Cow<'static, str> {
        match self {
            CpuWidgetColumn::Cpu => "CPU".into(),
            CpuWidgetColumn::Use { .. } => "Use".into(),
        }
    }
}

pub enum CpuWidgetTableData {
    All,
    Entry { data_type: CpuDataType, usage: f32 },
}

impl CpuWidgetTableData {
    pub fn from_cpu_data(data: &CpuData) -> CpuWidgetTableData {
        CpuWidgetTableData::Entry {
            data_type: data.data_type,
            usage: data.usage,
        }
    }
}

/// Returns a formatted CPU usage string based on the given usage value, whether
/// to show a decimal, and the available width.
///
/// Note that if there is not enough space to show the percentage sign or decimal place,
/// they will be truncated first rather than hiding the whole thing. If the actual whole
/// number value itself is unavailable, then it will entirely be hidden (and return [`None`]).
fn cpu_usage_text(usage: f32, show_decimal: bool, width: u16) -> Option<String> {
    let rounded_usage = if show_decimal {
        (usage * 10.0).round() / 10.0
    } else {
        usage.round()
    };

    let whole_usage_text_len = if rounded_usage >= 100.0 {
        3
    } else if rounded_usage >= 10.0 {
        2
    } else {
        1
    };

    // +1 to account for percentage sign, and +2 more if showing decimal for the dot and the digit.
    let text_length = whole_usage_text_len + if show_decimal { 3 } else { 1 };
    let width = usize::from(width);

    match text_length.cmp(&width) {
        Ordering::Less | Ordering::Equal => {
            if show_decimal {
                Some(format!("{rounded_usage:.1}%"))
            } else {
                Some(format!("{rounded_usage:.0}%"))
            }
        }
        Ordering::Greater => match text_length - width {
            1 => {
                // If it's 1 character off, just drop the percentage sign.
                if show_decimal {
                    Some(format!("{rounded_usage:.1}"))
                } else {
                    Some(format!("{rounded_usage:.0}"))
                }
            }
            2..=3 if show_decimal => {
                // If it's 2 to 3 characters off and decimal, then we just drop the decimal + percentage sign.
                Some(format!("{rounded_usage:.0}"))
            }
            _ => None,
        },
    }
}

impl DataToCell<CpuWidgetColumn> for CpuWidgetTableData {
    fn to_cell_text(
        &self, column: &CpuWidgetColumn, calculated_width: NonZeroU16,
    ) -> Option<Cow<'static, str>> {
        const CPU_TRUNCATE_BREAKPOINT: u16 = 5;

        let calculated_width = calculated_width.get();

        // This is a bit of a hack, but apparently we can avoid having to do any
        // fancy checks of showing the "All" on a specific column if the
        // other is hidden by just always showing it on the CPU (first)
        // column - if there isn't room for it, it will just collapse
        // down.
        //
        // This is the same for the use percentages - we prefer showing them,
        // but hide them if even the whole number is too small.
        match &self {
            CpuWidgetTableData::All => match column {
                CpuWidgetColumn::Cpu => Some("All".into()),
                CpuWidgetColumn::Use { .. } => None,
            },
            CpuWidgetTableData::Entry {
                data_type,
                usage: last_entry,
            } => {
                if calculated_width == 0 {
                    None
                } else {
                    match column {
                        CpuWidgetColumn::Cpu => match data_type {
                            CpuDataType::Avg => Some("AVG".into()),
                            CpuDataType::Cpu(index) => {
                                let index_str = index.to_string();
                                let text = if calculated_width < CPU_TRUNCATE_BREAKPOINT {
                                    index_str.into()
                                } else {
                                    concat_string!("CPU", index_str).into()
                                };

                                Some(text)
                            }
                        },
                        CpuWidgetColumn::Use { show_decimal } => {
                            cpu_usage_text(*last_entry, *show_decimal, calculated_width)
                                .map(Cow::Owned)
                        }
                    }
                }
            }
        }
    }

    #[inline(always)]
    fn style_row<'a>(&self, row: Row<'a>, painter: &Painter) -> Row<'a> {
        let style = match self {
            CpuWidgetTableData::All => painter.styles.all_cpu_colour,
            CpuWidgetTableData::Entry {
                data_type,
                usage: _,
            } => match data_type {
                CpuDataType::Avg => painter.styles.avg_cpu_colour,
                CpuDataType::Cpu(index) => {
                    painter.styles.cpu_colour_styles[index % painter.styles.cpu_colour_styles.len()]
                }
            },
        };

        row.style(style)
    }

    fn column_widths<C: DataTableColumn<CpuWidgetColumn>>(
        _data: &[Self], _columns: &[C],
    ) -> Vec<u16>
    where
        Self: Sized,
    {
        vec![1, 3]
    }
}

pub struct CpuWidgetState {
    pub graph: PercentTimeGraph,
    pub is_legend_hidden: bool,
    pub table: DataTable<CpuWidgetTableData, CpuWidgetColumn>,
    pub force_update_data: bool,
}

impl CpuWidgetState {
    pub(crate) fn new(
        config: &AppConfigFields, default_selection: CpuDefault, autohide_timer: Option<Instant>,
        colours: &Styles,
    ) -> Self {
        let ts_config = TimeseriesConfig {
            time_interval: config.time_interval,
            retention_ms: config.retention_ms,
            autohide_time: config.autohide_time,
            default_time_value: config.default_time_value,
        };
        let columns = [
            Column::soft(CpuWidgetColumn::Cpu, Some(0.5)),
            Column::soft(
                CpuWidgetColumn::Use {
                    show_decimal: config.show_cpu_decimal,
                },
                Some(0.5),
            ),
        ];

        let props = DataTableProps {
            title: None,
            table_gap: config.table_gap,
            left_to_right: false,
            is_basic: false,
            show_table_scroll_position: false, // TODO: Should this be possible?
            show_table_scroll_bar: config.show_table_scroll_bar,
            show_current_entry_when_unfocused: true,
        };

        let styling = DataTableStyling::from_palette(colours);
        let mut table = DataTable::new(columns, props, styling);
        match default_selection {
            CpuDefault::All => {}
            CpuDefault::Average if !config.show_average_cpu => {}
            CpuDefault::Average => {
                table = table.first_draw_index(1);
            }
        }

        CpuWidgetState {
            graph: PercentTimeGraph::new(ts_config, autohide_timer),
            is_legend_hidden: false,
            table,
            force_update_data: false,
        }
    }

    /// Forces an update of the data stored.
    #[inline]
    pub fn force_data_update(&mut self) {
        self.force_update_data = true;
    }

    pub fn set_legend_data(&mut self, data: &[CpuData]) {
        self.table.set_data(
            std::iter::once(CpuWidgetTableData::All)
                .chain(data.iter().map(CpuWidgetTableData::from_cpu_data))
                .collect(),
        );
        self.force_update_data = false;
    }
}

#[cfg(test)]
mod tests {
    use super::cpu_usage_text;

    #[test]
    fn cpu_usage_text_keeps_percent_if_fits() {
        assert_eq!(cpu_usage_text(100.0, false, 4).as_deref(), Some("100%"));
        assert_eq!(cpu_usage_text(100.0, true, 6).as_deref(), Some("100.0%"));

        assert_eq!(cpu_usage_text(99.0, false, 4).as_deref(), Some("99%"));
        assert_eq!(cpu_usage_text(99.0, true, 5).as_deref(), Some("99.0%"));

        assert_eq!(cpu_usage_text(9.0, false, 4).as_deref(), Some("9%"));
        assert_eq!(cpu_usage_text(9.0, true, 5).as_deref(), Some("9.0%"));
    }

    #[test]
    fn cpu_usage_text_drops_decimal_and_percent_if_needed() {
        assert_eq!(cpu_usage_text(100.0, false, 3).as_deref(), Some("100"));
        assert_eq!(cpu_usage_text(100.0, true, 3).as_deref(), Some("100"));
        assert_eq!(cpu_usage_text(100.0, true, 4).as_deref(), Some("100"));

        assert_eq!(cpu_usage_text(99.0, false, 2).as_deref(), Some("99"));
        assert_eq!(cpu_usage_text(99.0, true, 2).as_deref(), Some("99"));

        assert_eq!(cpu_usage_text(9.0, false, 1).as_deref(), Some("9"));
        assert_eq!(cpu_usage_text(9.0, true, 1).as_deref(), Some("9"));
    }

    #[test]
    fn cpu_usage_text_handles_values_that_round_up_to_an_extra_digit() {
        assert_eq!(cpu_usage_text(99.96, false, 3).as_deref(), Some("100"));
        assert_eq!(cpu_usage_text(99.96, false, 2), None);

        assert_eq!(cpu_usage_text(99.96, true, 3).as_deref(), Some("100"));
        assert_eq!(cpu_usage_text(99.96, true, 4).as_deref(), Some("100"));
        assert_eq!(cpu_usage_text(99.96, true, 5).as_deref(), Some("100.0"));
        assert_eq!(cpu_usage_text(99.96, true, 6).as_deref(), Some("100.0%"));
        assert_eq!(cpu_usage_text(99.96, true, 2), None);
    }

    #[test]
    fn cpu_usage_text_keeps_rounded_decimal_boundary_consistent_across_widths() {
        assert_eq!(cpu_usage_text(9.95, true, 5).as_deref(), Some("10.0%"));
        assert_eq!(cpu_usage_text(9.95, true, 4).as_deref(), Some("10.0"));
        assert_eq!(cpu_usage_text(9.95, true, 3).as_deref(), Some("10"));
    }

    #[test]
    fn cpu_usage_text_hides_values_if_too_wide() {
        assert_eq!(cpu_usage_text(100.0, false, 2), None);
        assert_eq!(cpu_usage_text(100.0, true, 2), None);

        assert_eq!(cpu_usage_text(99.0, false, 1), None);
        assert_eq!(cpu_usage_text(99.0, true, 1), None);

        assert_eq!(cpu_usage_text(9.0, false, 0), None);
        assert_eq!(cpu_usage_text(9.0, true, 0), None);
    }
}
