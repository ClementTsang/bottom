use std::{borrow::Cow, num::NonZeroU16, time::Instant};

use concat_string::concat_string;
use tui::{style::Style, widgets::Row};

use crate::{
    app::AppConfigFields,
    canvas::{
        components::data_table::{
            Column, ColumnHeader, DataTable, DataTableColumn, DataTableProps, DataTableStyling,
            DataToCell,
        },
        styling::CanvasStyles,
        Painter,
    },
    data_collection::cpu::CpuDataType,
    data_conversion::CpuWidgetData,
    options::config::cpu::CpuDefault,
};

#[derive(Default)]
pub struct CpuWidgetStyling {
    pub all: Style,
    pub avg: Style,
    pub entries: Vec<Style>,
}

impl CpuWidgetStyling {
    fn from_colours(colours: &CanvasStyles) -> Self {
        let entries = if colours.cpu_colour_styles.is_empty() {
            vec![Style::default()]
        } else {
            colours.cpu_colour_styles.clone()
        };

        Self {
            all: colours.all_colour_style,
            avg: colours.avg_colour_style,
            entries,
        }
    }
}

pub enum CpuWidgetColumn {
    CPU,
    Use,
}

impl ColumnHeader for CpuWidgetColumn {
    fn text(&self) -> Cow<'static, str> {
        match self {
            CpuWidgetColumn::CPU => "CPU".into(),
            CpuWidgetColumn::Use => "Use".into(),
        }
    }
}

pub enum CpuWidgetTableData {
    All,
    Entry {
        data_type: CpuDataType,
        last_entry: f64,
    },
}

impl CpuWidgetTableData {
    pub fn from_cpu_widget_data(data: &CpuWidgetData) -> CpuWidgetTableData {
        match data {
            CpuWidgetData::All => CpuWidgetTableData::All,
            CpuWidgetData::Entry {
                data_type,
                data: _,
                last_entry,
            } => CpuWidgetTableData::Entry {
                data_type: *data_type,
                last_entry: *last_entry,
            },
        }
    }
}

impl DataToCell<CpuWidgetColumn> for CpuWidgetTableData {
    fn to_cell(
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
                CpuWidgetColumn::CPU => Some("All".into()),
                CpuWidgetColumn::Use => None,
            },
            CpuWidgetTableData::Entry {
                data_type,
                last_entry,
            } => {
                if calculated_width == 0 {
                    None
                } else {
                    match column {
                        CpuWidgetColumn::CPU => match data_type {
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
            CpuWidgetTableData::All => painter.colours.all_cpu_colour,
            CpuWidgetTableData::Entry {
                data_type,
                last_entry: _,
            } => match data_type {
                CpuDataType::Avg => painter.colours.avg_cpu_colour,
                CpuDataType::Cpu(index) => {
                    painter.colours.cpu_colour_styles
                        [index % painter.colours.cpu_colour_styles.len()]
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
    pub show_avg: bool,
    pub autohide_timer: Option<Instant>,
    pub table: DataTable<CpuWidgetTableData, CpuWidgetColumn>,
    pub styling: CpuWidgetStyling,
}

impl CpuWidgetState {
    pub fn new(
        config: &AppConfigFields, default_selection: CpuDefault, current_display_time: u64,
        autohide_timer: Option<Instant>, colours: &CanvasStyles,
    ) -> Self {
        const COLUMNS: [Column<CpuWidgetColumn>; 2] = [
            Column::soft(CpuWidgetColumn::CPU, Some(0.5)),
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

        let styling = DataTableStyling::from_colours(colours);
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
            show_avg: config.show_average_cpu,
            autohide_timer,
            table,
            styling: CpuWidgetStyling::from_colours(colours),
        }
    }

    pub fn update_table(&mut self, data: &[CpuWidgetData]) {
        self.table.set_data(
            data.iter()
                .map(CpuWidgetTableData::from_cpu_widget_data)
                .collect(),
        );
    }
}
