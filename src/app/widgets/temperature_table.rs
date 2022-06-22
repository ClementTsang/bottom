use std::{borrow::Cow, cmp::max};

use concat_string::concat_string;
use kstring::KString;
use tui::text::Text;

use crate::{
    app::{data_harvester::temperature::TemperatureType, AppConfigFields},
    canvas::canvas_colours::CanvasColours,
    components::data_table::{
        Column, ColumnHeader, DataTable, DataTableColumn, DataTableProps, DataTableStyling,
        DataToCell,
    },
    utils::gen_util::truncate_text,
};

#[derive(Clone)]
pub struct TempWidgetData {
    pub sensor: KString,
    pub temperature_value: u64,
    pub temperature_type: TemperatureType,
}

pub enum TempWidgetColumn {
    Sensor,
    Temp,
}

impl ColumnHeader for TempWidgetColumn {
    fn text(&self) -> Cow<'static, str> {
        match self {
            TempWidgetColumn::Sensor => "Sensor".into(),
            TempWidgetColumn::Temp => "Temp".into(),
        }
    }
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

impl DataToCell<TempWidgetColumn> for TempWidgetData {
    fn to_cell<'a>(&'a self, column: &TempWidgetColumn, calculated_width: u16) -> Option<Text<'a>> {
        Some(match column {
            TempWidgetColumn::Sensor => truncate_text(&self.sensor, calculated_width),
            TempWidgetColumn::Temp => truncate_text(&self.temperature(), calculated_width),
        })
    }

    fn column_widths<C: DataTableColumn<TempWidgetColumn>>(
        data: &[TempWidgetData], _columns: &[C],
    ) -> Vec<u16>
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
    pub table: DataTable<TempWidgetData, TempWidgetColumn>,
}

impl TempWidgetState {
    pub fn new(config: &AppConfigFields, colours: &CanvasColours) -> Self {
        const COLUMNS: [Column<TempWidgetColumn>; 2] = [
            Column::soft(TempWidgetColumn::Sensor, Some(0.8)),
            Column::soft(TempWidgetColumn::Temp, None),
        ];

        let props = DataTableProps {
            title: Some(" Temperatures ".into()),
            table_gap: config.table_gap,
            left_to_right: false,
            is_basic: config.use_basic_mode,
            show_table_scroll_position: config.show_table_scroll_position,
            show_current_entry_when_unfocused: false,
        };

        let styling = DataTableStyling::from_colours(colours);

        Self {
            table: DataTable::new(COLUMNS, props, styling),
        }
    }
}
