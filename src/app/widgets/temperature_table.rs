use std::cmp::max;

use concat_string::concat_string;
use kstring::KString;
use tui::widgets::Row;

use crate::{
    app::{data_harvester::temperature::TemperatureType, AppConfigFields},
    canvas::canvas_colours::CanvasColours,
    components::data_table::{Column, DataTable, DataTableProps, DataTableStyling, ToDataRow},
    utils::gen_util::truncate_text,
};

pub struct TempWidgetData {
    pub sensor: KString,
    pub temperature_value: u64,
    pub temperature_type: TemperatureType,
}

impl TempWidgetData {
    pub fn temperature(&self) -> KString {
        let temp_val = self.temperature_value.to_string();
        let temp_type = match self.temperature_type {
            TemperatureType::Celsius => "°C",
            TemperatureType::Kelvin => "K",
            TemperatureType::Fahrenheit => "°F",
        };
        concat_string!(temp_val, temp_type).into()
    }
}

impl ToDataRow for TempWidgetData {
    fn to_data_row<'a>(&self, widths: &[u16]) -> Row<'a> {
        Row::new(vec![
            truncate_text(self.sensor.clone().into_cow_str(), widths[0].into()),
            truncate_text(self.temperature().into_cow_str(), widths[1].into()),
        ])
    }

    fn column_widths(data: &[TempWidgetData]) -> Vec<u16>
    where
        Self: Sized,
    {
        let mut widths = vec![0; 2];

        data.iter().for_each(|row| {
            widths[0] = max(widths[0], row.sensor.len() as u16);
            widths[1] = max(widths[1], row.temperature().len() as u16);
        });

        widths
    }
}

pub struct TempWidgetState {
    pub table: DataTable<TempWidgetData>,
}

impl TempWidgetState {
    pub fn new(config: &AppConfigFields, colours: &CanvasColours) -> Self {
        const COLUMNS: [Column<&str>; 2] = [
            Column::soft("Sensor", Some(0.8)),
            Column::soft("Temp", None),
        ];

        let props = DataTableProps {
            title: Some(" Temperatures ".into()),
            table_gap: config.table_gap,
            left_to_right: false,
            is_basic: config.use_basic_mode,
            show_table_scroll_position: config.show_table_scroll_position,
            show_current_entry_when_unfocused: false,
        };

        let styling = DataTableStyling {
            header_style: colours.table_header_style,
            border_style: colours.border_style,
            highlighted_border_style: colours.highlighted_border_style,
            text_style: colours.text_style,
            highlighted_text_style: colours.currently_selected_text_style,
            title_style: colours.widget_title_style,
        };

        Self {
            table: DataTable::new(COLUMNS, props, styling),
        }
    }
}
