use std::time::Instant;

use concat_string::concat_string;

use tui::{style::Style, widgets::Row};

use crate::{
    app::{data_harvester::cpu::CpuDataType, AppConfigFields},
    canvas::canvas_colours::CanvasColours,
    components::data_table::{DataTable, DataTableColumn, DataTableInner, DataTableProps},
    data_conversion::CpuWidgetData,
    utils::gen_util::truncate_text,
};

#[derive(Default)]
pub struct CpuWidgetStyling {
    pub all: Style,
    pub avg: Style,
    pub entries: Vec<Style>,
}

pub struct CpuWidgetInner {
    styling: CpuWidgetStyling,
}

impl DataTableInner<CpuWidgetData> for CpuWidgetInner {
    fn to_data_row<'a>(&self, data: &'a CpuWidgetData, columns: &[DataTableColumn]) -> Row<'a> {
        match data {
            CpuWidgetData::All => Row::new(vec![truncate_text(
                "All".into(),
                columns[0].calculated_width.into(),
            )])
            .style(self.styling.all),
            CpuWidgetData::Entry {
                data_type,
                data: _,
                last_entry,
            } => {
                let (entry_text, style) = match data_type {
                    CpuDataType::Avg => (
                        truncate_text("AVG".into(), columns[0].calculated_width.into()),
                        self.styling.avg,
                    ),
                    CpuDataType::Cpu(index) => {
                        let index_str = index.to_string();
                        let width = columns[0].calculated_width;
                        let text = if width < 5 {
                            truncate_text(index_str.into(), width.into())
                        } else {
                            truncate_text(concat_string!("CPU", index_str).into(), width.into())
                        };

                        (
                            text,
                            self.styling.entries[index % self.styling.entries.len()], // Theoretically never empty by initialization.
                        )
                    }
                };

                Row::new(vec![
                    entry_text,
                    truncate_text(
                        format!("{:.0}%", last_entry.round()).into(),
                        columns[1].calculated_width.into(),
                    ),
                ])
                .style(style)
            }
        }
    }

    fn column_widths(&self, _data: &[CpuWidgetData]) -> Vec<u16>
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
    pub table: DataTable<CpuWidgetData, CpuWidgetInner>,
}

impl CpuWidgetState {
    pub fn new(
        config: &AppConfigFields, current_display_time: u64, autohide_timer: Option<Instant>,
        colours: &CanvasColours,
    ) -> Self {
        const COLUMNS: [DataTableColumn; 2] = [
            DataTableColumn::soft("CPU", Some(0.5)),
            DataTableColumn::soft("Use%", Some(0.5)),
        ];

        let props = DataTableProps {
            title: None,
            table_gap: config.table_gap,
            left_to_right: false,
            is_basic: false,
            show_table_scroll_position: false, // TODO: Should this be possible?
            show_current_entry_when_unfocused: true,
        };

        CpuWidgetState {
            current_display_time,
            is_legend_hidden: false,
            show_avg: config.show_average_cpu,
            autohide_timer,
            table: DataTable::new(
                COLUMNS,
                props,
                CpuWidgetInner {
                    styling: CpuWidgetStyling {
                        all: colours.all_colour_style,
                        avg: colours.avg_colour_style,
                        entries: if colours.cpu_colour_styles.is_empty() {
                            vec![Style::default()]
                        } else {
                            colours.cpu_colour_styles.clone()
                        },
                    },
                },
            ),
        }
    }
}
