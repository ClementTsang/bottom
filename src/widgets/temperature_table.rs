use std::{borrow::Cow, cmp::max};

use concat_string::concat_string;
use kstring::KString;
use tui::text::Text;

use crate::{
    app::{data_harvester::temperature::TemperatureType, AppConfigFields},
    canvas::canvas_styling::CanvasColours,
    components::data_table::{
        ColumnHeader, DataTableColumn, DataTableProps, DataTableStyling, DataToCell, SortColumn,
        SortDataTable, SortDataTableProps, SortOrder, SortsRow,
    },
    utils::gen_util::{sort_partial_fn, truncate_to_text},
};

#[derive(Clone, Debug)]
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
            TempWidgetColumn::Sensor => "Sensor(s)".into(),
            TempWidgetColumn::Temp => "Temp(t)".into(),
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
        if calculated_width == 0 {
            return None;
        }

        Some(match column {
            TempWidgetColumn::Sensor => truncate_to_text(&self.sensor, calculated_width),
            TempWidgetColumn::Temp => truncate_to_text(&self.temperature(), calculated_width),
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

impl SortsRow for TempWidgetColumn {
    type DataType = TempWidgetData;

    fn sort_data(&self, data: &mut [Self::DataType], descending: bool) {
        match self {
            TempWidgetColumn::Sensor => {
                data.sort_by(move |a, b| sort_partial_fn(descending)(&a.sensor, &b.sensor));
            }
            TempWidgetColumn::Temp => {
                data.sort_by(|a, b| {
                    sort_partial_fn(descending)(a.temperature_value, b.temperature_value)
                });
            }
        }
    }
}

pub struct TempWidgetState {
    pub table: SortDataTable<TempWidgetData, TempWidgetColumn>,
    pub force_update_data: bool,
}

impl TempWidgetState {
    pub fn new(config: &AppConfigFields, colours: &CanvasColours) -> Self {
        let columns = [
            SortColumn::soft(TempWidgetColumn::Sensor, Some(0.8)),
            SortColumn::soft(TempWidgetColumn::Temp, None).default_descending(),
        ];

        let props = SortDataTableProps {
            inner: DataTableProps {
                title: Some(" Temperatures ".into()),
                table_gap: config.table_gap,
                left_to_right: false,
                is_basic: config.use_basic_mode,
                show_table_scroll_position: config.show_table_scroll_position,
                show_current_entry_when_unfocused: false,
            },
            sort_index: 0,
            order: SortOrder::Ascending,
        };

        let styling = DataTableStyling::from_colours(colours);

        Self {
            table: SortDataTable::new_sortable(columns, props, styling),
            force_update_data: false,
        }
    }

    /// Forces an update of the data stored.
    #[inline]
    pub fn force_data_update(&mut self) {
        self.force_update_data = true;
    }

    pub fn ingest_data(&mut self, data: &[TempWidgetData]) {
        let mut data = data.to_vec();
        if let Some(column) = self.table.columns.get(self.table.sort_index()) {
            column.sort_by(&mut data, self.table.order());
        }
        self.table.set_data(data);
    }
}
