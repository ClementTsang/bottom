use std::time::Instant;

use concat_string::concat_string;

use tui::{style::Style, widgets::Row};

use crate::{
    app::{data_harvester::cpu::CpuDataType, AppConfigFields},
    canvas::canvas_colours::CanvasColours,
    components::data_table::{Column, DataTable, DataTableProps, DataTableStyling, ToDataRow},
    data_conversion::{CpuWidgetData, CpuWidgetDataType},
    utils::gen_util::truncate_text,
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

impl ToDataRow for CpuWidgetData {
    fn to_data_row<'a>(&self, widths: &[u16]) -> Row<'a> {
        // FIXME: Adjust based on column widths
        match &self.data {
            CpuWidgetDataType::All => Row::new(vec![truncate_text("All".into(), widths[0].into())]),
            CpuWidgetDataType::Entry {
                data_type,
                data: _,
                last_entry,
            } => {
                let entry_text = match data_type {
                    CpuDataType::Avg => truncate_text("AVG".into(), widths[0].into()),
                    CpuDataType::Cpu(index) => {
                        let index_str = index.to_string();
                        let width = widths[0].into();
                        let text = if width < 5 {
                            truncate_text(index_str.into(), width)
                        } else {
                            truncate_text(concat_string!("CPU", index_str).into(), width)
                        };

                        text
                    }
                };

                Row::new(vec![
                    entry_text,
                    truncate_text(
                        format!("{:.0}%", last_entry.round()).into(),
                        widths[1].into(),
                    ),
                ])
            }
            .style(self.style),
        }
    }

    fn column_widths(_data: &[CpuWidgetData]) -> Vec<u16>
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
    pub table: DataTable<CpuWidgetData>,
    pub styling: CpuWidgetStyling,
}

impl CpuWidgetState {
    pub fn new(
        config: &AppConfigFields, current_display_time: u64, autohide_timer: Option<Instant>,
        colours: &CanvasColours,
    ) -> Self {
        const COLUMNS: [Column<&str>; 2] = [
            Column::soft("CPU", Some(0.5)),
            Column::soft("Use%", Some(0.5)),
        ];

        let props = DataTableProps {
            title: None,
            table_gap: config.table_gap,
            left_to_right: false,
            is_basic: false,
            show_table_scroll_position: false, // TODO: Should this be possible?
            show_current_entry_when_unfocused: true,
        };

        let styling = DataTableStyling {
            header_style: colours.table_header_style,
            border_style: colours.border_style,
            highlighted_border_style: colours.highlighted_border_style,
            text_style: colours.text_style,
            highlighted_text_style: colours.currently_selected_text_style,
            title_style: colours.widget_title_style,
        };

        CpuWidgetState {
            current_display_time,
            is_legend_hidden: false,
            show_avg: config.show_average_cpu,
            autohide_timer,
            table: DataTable::new(COLUMNS, props, styling),
            styling: CpuWidgetStyling::from_colours(colours),
        }
    }
}
