use std::{borrow::Cow, time::Instant};

use concat_string::concat_string;
use tui::{style::Style, text::Text, widgets::Row};

use crate::{
    app::{data_harvester::cpu::CpuDataType, AppConfigFields},
    canvas::{canvas_styling::CanvasColours, Painter},
    components::data_table::{
        Column, ColumnHeader, DataTable, DataTableColumn, DataTableProps, DataTableStyling,
        DataToCell,
    },
    data_conversion::CpuWidgetData,
    utils::gen_util::truncate_to_text,
};

#[derive(Default)]
pub struct CpuWidgetStyling {
    pub all: Style,
    pub avg: Style,
    pub entries: Vec<Style>,
}

impl CpuWidgetStyling {
    fn from_colours(colours: &CanvasColours) -> Self {
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
            CpuWidgetColumn::Use => "Use%".into(),
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
    fn to_cell<'a>(&'a self, column: &CpuWidgetColumn, calculated_width: u16) -> Option<Text<'a>> {
        const CPU_TRUNCATE_BREAKPOINT: u16 = 5;

        // This is a bit of a hack, but apparently we can avoid having to do any fancy checks
        // of showing the "All" on a specific column if the other is hidden by just always
        // showing it on the CPU (first) column - if there isn't room for it, it will just collapse
        // down.
        //
        // This is the same for the use percentages - we just *always* show them, and *always* hide the CPU column if
        // it is too small.
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
                            CpuDataType::Avg => Some(truncate_to_text("AVG", calculated_width)),
                            CpuDataType::Cpu(index) => {
                                let index_str = index.to_string();
                                let text = if calculated_width < CPU_TRUNCATE_BREAKPOINT {
                                    truncate_to_text(&index_str, calculated_width)
                                } else {
                                    truncate_to_text(
                                        &concat_string!("CPU", index_str),
                                        calculated_width,
                                    )
                                };

                                Some(text)
                            }
                        },
                        CpuWidgetColumn::Use => Some(truncate_to_text(
                            &format!("{:.0}%", last_entry.round()),
                            calculated_width,
                        )),
                    }
                }
            }
        }
    }

    #[inline(always)]
    fn style_row<'a>(&self, row: Row<'a>, painter: &Painter) -> Row<'a> {
        let style = match self {
            CpuWidgetTableData::All => painter.colours.all_colour_style,
            CpuWidgetTableData::Entry {
                data_type,
                last_entry: _,
            } => match data_type {
                CpuDataType::Avg => painter.colours.avg_colour_style,
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
        config: &AppConfigFields, current_display_time: u64, autohide_timer: Option<Instant>,
        colours: &CanvasColours,
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

        CpuWidgetState {
            current_display_time,
            is_legend_hidden: false,
            show_avg: config.show_average_cpu,
            autohide_timer,
            table: DataTable::new(COLUMNS, props, styling),
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
