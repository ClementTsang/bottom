use std::cmp::max;

use concat_string::concat_string;
use kstring::KString;
use tui::widgets::Row;

use crate::{
    app::{data_harvester::temperature::TemperatureType, AppConfigFields},
    components::data_table::{DataColumn, DataTable, DataTableProps, ToDataRow},
    utils::gen_util::truncate_text,
};

pub struct TempWidgetData {
    pub sensor: KString,
    pub temperature_value: u64,
    pub temperature_type: TemperatureType,
}

impl TempWidgetData {
    pub fn temperature(&self) -> KString {
        concat_string!(
            self.temperature_value.to_string(),
            match self.temperature_type {
                TemperatureType::Celsius => "°C",
                TemperatureType::Kelvin => "K",
                TemperatureType::Fahrenheit => "°F",
            }
        )
        .into()
    }
}

impl ToDataRow for TempWidgetData {
    fn to_data_row<'a>(&'a self, columns: &[DataColumn]) -> Row<'a> {
        Row::new(vec![
            truncate_text(
                self.sensor.clone().into_cow_str(),
                columns[0].calculated_width.into(),
            ),
            truncate_text(
                self.temperature().into_cow_str(),
                columns[1].calculated_width.into(),
            ),
        ])
    }

    fn column_widths(data: &[Self]) -> Vec<u16>
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
    pub fn new(config: &AppConfigFields) -> Self {
        const COLUMNS: [DataColumn; 2] = [
            DataColumn::soft("Sensor", Some(0.8)),
            DataColumn::soft("Temp", None),
        ];

        let props = DataTableProps {
            title: Some(" Temperatures ".into()),
            table_gap: config.table_gap,
            left_to_right: false,
            is_basic: config.use_basic_mode,
            show_table_scroll_position: config.show_table_scroll_position,
        };

        Self {
            table: DataTable::new(COLUMNS.to_vec(), props),
        }
    }
}
