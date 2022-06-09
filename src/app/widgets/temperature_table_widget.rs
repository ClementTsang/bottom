use std::cmp::max;

use concat_string::concat_string;
use kstring::KString;
use tui::widgets::Row;

use crate::{
    app::{data_harvester::temperature::TemperatureType, AppConfigFields},
    components::data_table::{DataTable, DataTableColumn, DataTableInner, DataTableProps},
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

pub struct TempWidgetInner;

impl DataTableInner<TempWidgetData> for TempWidgetInner {
    fn to_data_row<'a>(&self, data: &'a TempWidgetData, columns: &[DataTableColumn]) -> Row<'a> {
        Row::new(vec![
            truncate_text(
                data.sensor.clone().into_cow_str(),
                columns[0].calculated_width.into(),
            ),
            truncate_text(
                data.temperature().into_cow_str(),
                columns[1].calculated_width.into(),
            ),
        ])
    }

    fn column_widths(&self, data: &[TempWidgetData]) -> Vec<u16>
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
    pub table: DataTable<TempWidgetData, TempWidgetInner>,
}

impl TempWidgetState {
    pub fn new(config: &AppConfigFields) -> Self {
        const COLUMNS: [DataTableColumn; 2] = [
            DataTableColumn::soft("Sensor", Some(0.8)),
            DataTableColumn::soft("Temp", None),
        ];

        let props = DataTableProps {
            title: Some(" Temperatures ".into()),
            table_gap: config.table_gap,
            left_to_right: false,
            is_basic: config.use_basic_mode,
            show_table_scroll_position: config.show_table_scroll_position,
            show_current_entry_when_unfocused: false,
        };

        Self {
            table: DataTable::new(COLUMNS, props, TempWidgetInner {}),
        }
    }
}
