use std::{borrow::Cow, num::NonZeroU16, time::Instant};

use concat_string::concat_string;
use tui::widgets::Row;

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
    options::config::{cpu::CpuDefault, style::Styles},
};

pub enum CpuWidgetColumn {
    Cpu,
    Use,
}

impl ColumnHeader for CpuWidgetColumn {
    fn text(&self) -> Cow<'static, str> {
        match self {
            CpuWidgetColumn::Cpu => "CPU".into(),
            CpuWidgetColumn::Use => "Use".into(),
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

        // This is a bit of a hack, but apparently we can avoid having to do any fancy
        // checks of showing the "All" on a specific column if the other is
        // hidden by just always showing it on the CPU (first) column - if there
        // isn't room for it, it will just collapse down.
        //
        // This is the same for the use percentages - we just *always* show them, and
        // *always* hide the CPU column if it is too small.
        match &self {
            CpuWidgetTableData::All => match column {
                CpuWidgetColumn::Cpu => Some("All".into()),
                CpuWidgetColumn::Use => None,
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
                        CpuWidgetColumn::Use => Some(format!("{:.0}%", last_entry.round()).into()),
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
                    let index = *index as usize;
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
    pub current_display_time: u64,
    pub is_legend_hidden: bool,
    pub autohide_timer: Option<Instant>,
    pub table: DataTable<CpuWidgetTableData, CpuWidgetColumn>,
    pub force_update_data: bool,
}

impl CpuWidgetState {
    pub(crate) fn new(
        config: &AppConfigFields, default_selection: CpuDefault, current_display_time: u64,
        autohide_timer: Option<Instant>, colours: &Styles,
    ) -> Self {
        const COLUMNS: [Column<CpuWidgetColumn>; 2] = [
            Column::soft(CpuWidgetColumn::Cpu, Some(0.5)),
            Column::soft(CpuWidgetColumn::Use, Some(0.5)),
        ];

        let props = DataTableProps {
            title: None,
            table_gap: config.table_gap,
            left_to_right: false,
            is_basic: false,
            show_table_scroll_position: false, // TODO: Should this be possible?
            show_current_entry_when_unfocused: true,
        };

        let styling = DataTableStyling::from_palette(colours);
        let mut table = DataTable::new(COLUMNS, props, styling);
        match default_selection {
            CpuDefault::All => {}
            CpuDefault::Average if !config.show_average_cpu => {}
            CpuDefault::Average => {
                table = table.first_draw_index(1);
            }
        }

        CpuWidgetState {
            current_display_time,
            is_legend_hidden: false,
            autohide_timer,
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
