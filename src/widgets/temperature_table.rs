use std::{borrow::Cow, cmp::max, num::NonZeroU16};

use concat_string::concat_string;

use crate::{
    app::AppConfigFields,
    canvas::{
        components::data_table::{
            ColumnHeader, DataTableColumn, DataTableProps, DataTableStyling, DataToCell,
            SortColumn, SortDataTable, SortDataTableProps, SortOrder, SortsRow,
        },
        styling::CanvasStyling,
    },
    data_collection::temperature::TemperatureType,
    utils::general::sort_partial_fn,
};

#[derive(Clone, Debug)]
pub struct TempWidgetData {
    pub sensor: Cow<'static, str>,
    pub temperature_value: Option<u64>,
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
    pub fn temperature(&self) -> Cow<'static, str> {
        match self.temperature_value {
            Some(temp_val) => {
                let temp_type = match self.temperature_type {
                    TemperatureType::Celsius => "°C",
                    TemperatureType::Kelvin => "K",
                    TemperatureType::Fahrenheit => "°F",
                };
                concat_string!(temp_val.to_string(), temp_type).into()
            }
            None => "N/A".to_string().into(),
        }
    }
}

impl DataToCell<TempWidgetColumn> for TempWidgetData {
    fn to_cell(
        &self, column: &TempWidgetColumn, _calculated_width: NonZeroU16,
    ) -> Option<Cow<'static, str>> {
        Some(match column {
            TempWidgetColumn::Sensor => self.sensor.clone(),
            TempWidgetColumn::Temp => self.temperature(),
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
    pub fn new(config: &AppConfigFields, colours: &CanvasStyling) -> Self {
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

    /// Update the current table data.
    pub fn set_table_data(&mut self, data: &[TempWidgetData]) {
        let mut data = data.to_vec();
        if let Some(column) = self.table.columns.get(self.table.sort_index()) {
            column.sort_by(&mut data, self.table.order());
        }
        self.table.set_data(data);
    }
}
