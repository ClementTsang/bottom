use std::{borrow::Cow, num::NonZeroU16, time::Instant};

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

/// The full short label for a CPU entry, e.g. `"AVG"` or `"CPU3"`.
///
/// Shared between the classic side table and the in-chart legend so the naming
/// rule lives in exactly one place.
pub(crate) fn cpu_entry_name(data_type: CpuDataType) -> Cow<'static, str> {
    match data_type {
        CpuDataType::Avg => "AVG".into(),
        CpuDataType::Cpu(index) => concat_string!("CPU", index.to_string()).into(),
    }
}

/// The usage value for a CPU entry, honouring whether a decimal place is shown.
fn cpu_usage_value_str(usage: f32, show_decimal: bool) -> String {
    if show_decimal {
        format!("{usage:.1}")
    } else {
        format!("{usage:.0}")
    }
}

/// The usage string for a CPU entry, e.g. `"12%"` or `"8.4%"`.
pub(crate) fn cpu_usage_str(usage: f32, show_decimal: bool) -> String {
    concat_string!(cpu_usage_value_str(usage, show_decimal), "%")
}

/// The full label for a CPU entry in the in-chart legend, e.g. `"AVG  12%"`.
///
/// Uses a fixed-width usage column so entries line up, matching the memory
/// widget's legend.
pub(crate) fn cpu_legend_label(
    data_type: CpuDataType, usage: f32, show_decimal: bool,
) -> Cow<'static, str> {
    Cow::Owned(format!(
        "{} {:>3}%",
        cpu_entry_name(data_type),
        cpu_usage_value_str(usage, show_decimal)
    ))
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
        // This is the same for the use percentages - we just *always* show
        // them, and *always* hide the CPU column if it is too small.
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
                            CpuDataType::Avg => Some(cpu_entry_name(CpuDataType::Avg)),
                            CpuDataType::Cpu(index) => {
                                Some(if calculated_width < CPU_TRUNCATE_BREAKPOINT {
                                    index.to_string().into()
                                } else {
                                    cpu_entry_name(CpuDataType::Cpu(*index))
                                })
                            }
                        },
                        CpuWidgetColumn::Use { show_decimal } => {
                            Some(cpu_usage_str(*last_entry, *show_decimal).into())
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
mod test {
    use super::*;

    #[test]
    fn cpu_entry_names() {
        assert_eq!(cpu_entry_name(CpuDataType::Avg), "AVG");
        assert_eq!(cpu_entry_name(CpuDataType::Cpu(0)), "CPU0");
        assert_eq!(cpu_entry_name(CpuDataType::Cpu(12)), "CPU12");
    }

    #[test]
    fn cpu_usage_respects_decimal_setting() {
        assert_eq!(cpu_usage_str(12.4, false), "12%");
        assert_eq!(cpu_usage_str(12.4, true), "12.4%");
    }

    #[test]
    fn cpu_legend_label_aligns_and_respects_decimal_setting() {
        // The usage column is right-aligned to a fixed width so entries line up.
        assert_eq!(
            cpu_legend_label(CpuDataType::Cpu(3), 8.2, false),
            "CPU3   8%"
        );
        assert_eq!(cpu_legend_label(CpuDataType::Avg, 12.4, false), "AVG  12%");
        assert_eq!(
            cpu_legend_label(CpuDataType::Cpu(3), 8.2, true),
            "CPU3 8.2%"
        );
        assert_eq!(cpu_legend_label(CpuDataType::Avg, 100.0, false), "AVG 100%");
    }
}
