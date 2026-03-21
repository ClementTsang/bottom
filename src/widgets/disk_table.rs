use std::{borrow::Cow, cmp::max, num::NonZeroU16};

use serde::Deserialize;

use crate::{
    app::{AppConfigFields, data::StoredData},
    canvas::components::data_table::{
        ColumnHeader, DataTableColumn, DataTableProps, DataTableStyling, DataToCell, SortColumn,
        SortDataTable, SortDataTableProps, SortOrder, SortsRow,
    },
    options::config::style::Styles,
    utils::{
        conversion::dec_bytes_per_second_string, data_units::get_decimal_bytes,
        general::sort_partial_fn,
    },
};

#[derive(Clone, Debug)]
pub struct DiskWidgetData {
    pub name: String,
    pub mount_point: String,
    pub free_bytes: Option<u64>,
    pub used_bytes: Option<u64>,
    pub total_bytes: Option<u64>,
    pub summed_total_bytes: Option<u64>,
    pub io_read_rate_bytes: Option<u64>,
    pub io_write_rate_bytes: Option<u64>,
}

impl DiskWidgetData {
    fn total_space(&self) -> Cow<'static, str> {
        if let Some(total_bytes) = self.total_bytes {
            let converted_total_space = get_decimal_bytes(total_bytes);
            format!("{:.0}{}", converted_total_space.0, converted_total_space.1).into()
        } else {
            "N/A".into()
        }
    }

    fn free_space(&self) -> Cow<'static, str> {
        if let Some(free_bytes) = self.free_bytes {
            let converted_free_space = get_decimal_bytes(free_bytes);
            format!("{:.0}{}", converted_free_space.0, converted_free_space.1).into()
        } else {
            "N/A".into()
        }
    }

    fn used_space(&self) -> Cow<'static, str> {
        if let Some(used_bytes) = self.used_bytes {
            let converted_free_space = get_decimal_bytes(used_bytes);
            format!("{:.0}{}", converted_free_space.0, converted_free_space.1).into()
        } else {
            "N/A".into()
        }
    }

    fn free_percent(&self) -> Option<f64> {
        if let (Some(free_bytes), Some(summed_total_bytes)) =
            (self.free_bytes, self.summed_total_bytes)
        {
            if summed_total_bytes > 0 {
                Some(free_bytes as f64 / summed_total_bytes as f64 * 100_f64)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn used_percent(&self) -> Option<f64> {
        if let (Some(used_bytes), Some(summed_total_bytes)) =
            (self.used_bytes, self.summed_total_bytes)
        {
            if summed_total_bytes > 0 {
                Some(used_bytes as f64 / summed_total_bytes as f64 * 100_f64)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn io_read(&self) -> Cow<'static, str> {
        self.io_read_rate_bytes.map_or("N/A".into(), |r_rate| {
            dec_bytes_per_second_string(r_rate).into()
        })
    }

    fn io_write(&self) -> Cow<'static, str> {
        self.io_write_rate_bytes.map_or("N/A".into(), |w_rate| {
            dec_bytes_per_second_string(w_rate).into()
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "generate_schema",
    derive(schemars::JsonSchema, strum::VariantArray)
)]
pub enum DiskWidgetColumn {
    Disk,
    Mount,
    Used,
    Free,
    Total,
    UsedPercent,
    FreePercent,
    IoRead,
    IoWrite,
}

impl<'de> Deserialize<'de> for DiskWidgetColumn {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?.to_lowercase();
        match value.as_str() {
            "disk" => Ok(DiskWidgetColumn::Disk),
            "mount" => Ok(DiskWidgetColumn::Mount),
            "used" => Ok(DiskWidgetColumn::Used),
            "free" => Ok(DiskWidgetColumn::Free),
            "total" => Ok(DiskWidgetColumn::Total),
            "usedpercent" | "used%" => Ok(DiskWidgetColumn::UsedPercent),
            "freepercent" | "free%" => Ok(DiskWidgetColumn::FreePercent),
            "r/s" => Ok(DiskWidgetColumn::IoRead),
            "w/s" => Ok(DiskWidgetColumn::IoWrite),
            _ => Err(serde::de::Error::custom(
                "doesn't match any disk column name",
            )),
        }
    }
}

impl DiskWidgetColumn {
    /// An ugly hack to generate the JSON schema.
    #[cfg(feature = "generate_schema")]
    pub fn get_schema_names(&self) -> &[&'static str] {
        match self {
            DiskWidgetColumn::Disk => &["Disk"],
            DiskWidgetColumn::Mount => &["Mount"],
            DiskWidgetColumn::Used => &["Used"],
            DiskWidgetColumn::Free => &["Free"],
            DiskWidgetColumn::Total => &["Total"],
            DiskWidgetColumn::UsedPercent => &["Used%"],
            DiskWidgetColumn::FreePercent => &["Free%"],
            DiskWidgetColumn::IoRead => &["R/s", "Read", "Rps"],
            DiskWidgetColumn::IoWrite => &["W/s", "Write", "Wps"],
        }
    }
}

impl ColumnHeader for DiskWidgetColumn {
    fn text(&self) -> Cow<'static, str> {
        match self {
            DiskWidgetColumn::Disk => "Disk(d)",
            DiskWidgetColumn::Mount => "Mount(m)",
            DiskWidgetColumn::Used => "Used(u)",
            DiskWidgetColumn::Free => "Free(n)",
            DiskWidgetColumn::Total => "Total(t)",
            DiskWidgetColumn::UsedPercent => "Used%(p)",
            DiskWidgetColumn::FreePercent => "Free%",
            DiskWidgetColumn::IoRead => "R/s(r)",
            DiskWidgetColumn::IoWrite => "W/s(w)",
        }
        .into()
    }
}

impl DataToCell<DiskWidgetColumn> for DiskWidgetData {
    // FIXME: (points_rework_v1) Can we change the return type to 'a instead of 'static?
    fn to_cell_text(
        &self, column: &DiskWidgetColumn, _calculated_width: NonZeroU16,
    ) -> Option<Cow<'static, str>> {
        fn percent_string(value: Option<f64>) -> Cow<'static, str> {
            match value {
                Some(val) => format!("{val:.1}%").into(),
                None => "N/A".into(),
            }
        }

        let text = match column {
            DiskWidgetColumn::Disk => self.name.clone().into(),
            DiskWidgetColumn::Mount => self.mount_point.clone().into(),
            DiskWidgetColumn::Used => self.used_space(),
            DiskWidgetColumn::Free => self.free_space(),
            DiskWidgetColumn::UsedPercent => percent_string(self.used_percent()),
            DiskWidgetColumn::FreePercent => percent_string(self.free_percent()),
            DiskWidgetColumn::Total => self.total_space(),
            DiskWidgetColumn::IoRead => self.io_read(),
            DiskWidgetColumn::IoWrite => self.io_write(),
        };

        Some(text)
    }

    fn column_widths<C: DataTableColumn<DiskWidgetColumn>>(
        data: &[Self], _columns: &[C],
    ) -> Vec<u16>
    where
        Self: Sized,
    {
        let mut widths = vec![0; 7];

        data.iter().for_each(|row| {
            widths[0] = max(widths[0], row.name.len() as u16);
            widths[1] = max(widths[1], row.mount_point.len() as u16);
        });

        widths
    }
}

pub struct DiskTableWidget {
    pub table: SortDataTable<DiskWidgetData, DiskWidgetColumn>,
    pub force_update_data: bool,
}

impl SortsRow for DiskWidgetColumn {
    type DataType = DiskWidgetData;

    fn sort_data(&self, data: &mut [Self::DataType], descending: bool) {
        match self {
            DiskWidgetColumn::Disk => {
                data.sort_by(|a, b| sort_partial_fn(descending)(&a.name, &b.name));
            }
            DiskWidgetColumn::Mount => {
                data.sort_by(|a, b| sort_partial_fn(descending)(&a.mount_point, &b.mount_point));
            }
            DiskWidgetColumn::Used => {
                data.sort_by(|a, b| sort_partial_fn(descending)(&a.used_bytes, &b.used_bytes));
            }
            DiskWidgetColumn::UsedPercent => {
                data.sort_by(|a, b| {
                    sort_partial_fn(descending)(&a.used_percent(), &b.used_percent())
                });
            }
            DiskWidgetColumn::Free => {
                data.sort_by(|a, b| sort_partial_fn(descending)(&a.free_bytes, &b.free_bytes));
            }
            DiskWidgetColumn::FreePercent => {
                data.sort_by(|a, b| {
                    sort_partial_fn(descending)(&a.free_percent(), &b.free_percent())
                });
            }
            DiskWidgetColumn::Total => {
                data.sort_by(|a, b| sort_partial_fn(descending)(&a.total_bytes, &b.total_bytes));
            }
            DiskWidgetColumn::IoRead => {
                data.sort_by(|a, b| {
                    sort_partial_fn(descending)(&a.io_read_rate_bytes, &b.io_read_rate_bytes)
                });
            }
            DiskWidgetColumn::IoWrite => {
                data.sort_by(|a, b| {
                    sort_partial_fn(descending)(&a.io_write_rate_bytes, &b.io_write_rate_bytes)
                });
            }
        }
    }
}

const fn create_column(column_type: &DiskWidgetColumn) -> SortColumn<DiskWidgetColumn> {
    match column_type {
        DiskWidgetColumn::Disk => SortColumn::soft(DiskWidgetColumn::Disk, Some(0.2)),
        DiskWidgetColumn::Mount => SortColumn::soft(DiskWidgetColumn::Mount, Some(0.2)),
        DiskWidgetColumn::Used => SortColumn::hard(DiskWidgetColumn::Used, 8).default_descending(),
        DiskWidgetColumn::Free => SortColumn::hard(DiskWidgetColumn::Free, 8).default_descending(),
        DiskWidgetColumn::Total => {
            SortColumn::hard(DiskWidgetColumn::Total, 9).default_descending()
        }
        DiskWidgetColumn::UsedPercent => {
            SortColumn::hard(DiskWidgetColumn::UsedPercent, 9).default_descending()
        }
        DiskWidgetColumn::FreePercent => {
            SortColumn::hard(DiskWidgetColumn::FreePercent, 9).default_descending()
        }
        DiskWidgetColumn::IoRead => {
            SortColumn::hard(DiskWidgetColumn::IoRead, 10).default_descending()
        }
        DiskWidgetColumn::IoWrite => {
            SortColumn::hard(DiskWidgetColumn::IoWrite, 11).default_descending()
        }
    }
}

const fn default_disk_column_list() -> [DiskWidgetColumn; 8] {
    [
        DiskWidgetColumn::Disk,
        DiskWidgetColumn::Mount,
        DiskWidgetColumn::Used,
        DiskWidgetColumn::Free,
        DiskWidgetColumn::Total,
        DiskWidgetColumn::UsedPercent,
        DiskWidgetColumn::IoRead,
        DiskWidgetColumn::IoWrite,
    ]
}

const fn default_disk_columns() -> [SortColumn<DiskWidgetColumn>; 8] {
    [
        create_column(&DiskWidgetColumn::Disk),
        create_column(&DiskWidgetColumn::Mount),
        create_column(&DiskWidgetColumn::Used),
        create_column(&DiskWidgetColumn::Free),
        create_column(&DiskWidgetColumn::Total),
        create_column(&DiskWidgetColumn::UsedPercent),
        create_column(&DiskWidgetColumn::IoRead),
        create_column(&DiskWidgetColumn::IoWrite),
    ]
}

impl DiskTableWidget {
    pub fn new(
        config: &AppConfigFields, palette: &Styles, columns: Option<&[DiskWidgetColumn]>,
    ) -> Self {
        let props = SortDataTableProps {
            inner: DataTableProps {
                title: Some(" Disks ".into()),
                table_gap: config.table_gap,
                left_to_right: true,
                is_basic: config.use_basic_mode,
                show_table_scroll_position: config.show_table_scroll_position,
                show_current_entry_when_unfocused: false,
            },
            sort_index: match &config.default_disk_sort_column {
                Some(column) => {
                    // Must check that the column used exists. If not, fall back to 0.

                    let existing_columns = match columns {
                        Some(c) => c,
                        None => &default_disk_column_list(),
                    };

                    existing_columns
                        .iter()
                        .position(|c| c == column)
                        .unwrap_or_default()
                }
                None => 0,
            },
            order: SortOrder::Ascending,
        };

        let styling = DataTableStyling::from_palette(palette);

        match columns {
            Some(columns) => {
                let columns = columns.iter().map(create_column).collect::<Vec<_>>();
                Self {
                    table: SortDataTable::new_sortable(columns, props, styling),
                    force_update_data: false,
                }
            }
            None => Self {
                table: SortDataTable::new_sortable(default_disk_columns(), props, styling),
                force_update_data: false,
            },
        }
    }

    /// Forces an update of the data stored.
    #[inline]
    pub fn force_data_update(&mut self) {
        self.force_update_data = true;
    }

    /// Update the current table data.
    pub fn set_table_data(&mut self, data: &StoredData) {
        let mut data = data.disk_harvest.clone();

        if let Some(column) = self.table.columns.get(self.table.sort_index()) {
            column.sort_by(&mut data, self.table.order());
        }
        self.table.set_data(data);
        self.force_update_data = false;
    }

    pub fn set_index(&mut self, index: usize) {
        self.table.set_sort_index(index);
        self.force_data_update();
    }
}
