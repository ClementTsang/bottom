use std::{borrow::Cow, cmp::max, num::NonZeroU16};

use serde::Deserialize;

use crate::{
    app::{AppConfigFields, data::StoredData},
    canvas::components::data_table::{
        ColumnHeader, DataTableColumn, DataTableProps, DataTableStyling, DataToCell, SortColumn,
        SortDataTable, SortDataTableProps, SortOrder, SortsRow,
    },
    options::config::style::Styles,
    utils::{data_units::get_decimal_bytes, general::sort_partial_fn},
};

#[derive(Clone, Debug)]
pub struct DiskWidgetData {
    pub name: String,
    pub mount_point: String,
    pub free_bytes: Option<u64>,
    pub used_bytes: Option<u64>,
    pub total_bytes: Option<u64>,
    pub summed_total_bytes: Option<u64>,
    pub io_read: Cow<'static, str>,
    pub io_write: Cow<'static, str>,
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
}

#[derive(Debug, Clone)]
#[cfg_attr(
    feature = "generate_schema",
    derive(schemars::JsonSchema, strum::VariantArray)
)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub enum DiskColumn {
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

impl<'de> Deserialize<'de> for DiskColumn {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?.to_lowercase();
        match value.as_str() {
            "disk" => Ok(DiskColumn::Disk),
            "mount" => Ok(DiskColumn::Mount),
            "used" => Ok(DiskColumn::Used),
            "free" => Ok(DiskColumn::Free),
            "total" => Ok(DiskColumn::Total),
            "usedpercent" | "used%" => Ok(DiskColumn::UsedPercent),
            "freepercent" | "free%" => Ok(DiskColumn::FreePercent),
            "r/s" => Ok(DiskColumn::IoRead),
            "w/s" => Ok(DiskColumn::IoWrite),
            _ => Err(serde::de::Error::custom(
                "doesn't match any disk column name",
            )),
        }
    }
}

impl DiskColumn {
    /// An ugly hack to generate the JSON schema.
    #[cfg(feature = "generate_schema")]
    pub fn get_schema_names(&self) -> &[&'static str] {
        match self {
            DiskColumn::Disk => &["Disk"],
            DiskColumn::Mount => &["Mount"],
            DiskColumn::Used => &["Used"],
            DiskColumn::Free => &["Free"],
            DiskColumn::Total => &["Total"],
            DiskColumn::UsedPercent => &["Used%"],
            DiskColumn::FreePercent => &["Free%"],
            DiskColumn::IoRead => &["R/s", "Read", "Rps"],
            DiskColumn::IoWrite => &["W/s", "Write", "Wps"],
        }
    }
}

impl ColumnHeader for DiskColumn {
    fn text(&self) -> Cow<'static, str> {
        match self {
            DiskColumn::Disk => "Disk(d)",
            DiskColumn::Mount => "Mount(m)",
            DiskColumn::Used => "Used(u)",
            DiskColumn::Free => "Free(n)",
            DiskColumn::Total => "Total(t)",
            DiskColumn::UsedPercent => "Used%(p)",
            DiskColumn::FreePercent => "Free%",
            DiskColumn::IoRead => "R/s(r)",
            DiskColumn::IoWrite => "W/s(w)",
        }
        .into()
    }
}

impl DataToCell<DiskColumn> for DiskWidgetData {
    // FIXME: (points_rework_v1) Can we change the return type to 'a instead of 'static?
    fn to_cell_text(
        &self, column: &DiskColumn, _calculated_width: NonZeroU16,
    ) -> Option<Cow<'static, str>> {
        fn percent_string(value: Option<f64>) -> Cow<'static, str> {
            match value {
                Some(val) => format!("{val:.1}%").into(),
                None => "N/A".into(),
            }
        }

        let text = match column {
            DiskColumn::Disk => self.name.clone().into(),
            DiskColumn::Mount => self.mount_point.clone().into(),
            DiskColumn::Used => self.used_space(),
            DiskColumn::Free => self.free_space(),
            DiskColumn::UsedPercent => percent_string(self.used_percent()),
            DiskColumn::FreePercent => percent_string(self.free_percent()),
            DiskColumn::Total => self.total_space(),
            DiskColumn::IoRead => self.io_read.clone(),
            DiskColumn::IoWrite => self.io_write.clone(),
        };

        Some(text)
    }

    fn column_widths<C: DataTableColumn<DiskColumn>>(data: &[Self], _columns: &[C]) -> Vec<u16>
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
    pub table: SortDataTable<DiskWidgetData, DiskColumn>,
    pub force_update_data: bool,
}

impl SortsRow for DiskColumn {
    type DataType = DiskWidgetData;

    fn sort_data(&self, data: &mut [Self::DataType], descending: bool) {
        match self {
            DiskColumn::Disk => {
                data.sort_by(|a, b| sort_partial_fn(descending)(&a.name, &b.name));
            }
            DiskColumn::Mount => {
                data.sort_by(|a, b| sort_partial_fn(descending)(&a.mount_point, &b.mount_point));
            }
            DiskColumn::Used => {
                data.sort_by(|a, b| sort_partial_fn(descending)(&a.used_bytes, &b.used_bytes));
            }
            DiskColumn::UsedPercent => {
                data.sort_by(|a, b| {
                    sort_partial_fn(descending)(&a.used_percent(), &b.used_percent())
                });
            }
            DiskColumn::Free => {
                data.sort_by(|a, b| sort_partial_fn(descending)(&a.free_bytes, &b.free_bytes));
            }
            DiskColumn::FreePercent => {
                data.sort_by(|a, b| {
                    sort_partial_fn(descending)(&a.free_percent(), &b.free_percent())
                });
            }
            DiskColumn::Total => {
                data.sort_by(|a, b| sort_partial_fn(descending)(&a.total_bytes, &b.total_bytes));
            }
            DiskColumn::IoRead => {
                data.sort_by(|a, b| sort_partial_fn(descending)(&a.io_read, &b.io_read));
            }
            DiskColumn::IoWrite => {
                data.sort_by(|a, b| sort_partial_fn(descending)(&a.io_write, &b.io_write));
            }
        }
    }
}

const fn create_column(column_type: &DiskColumn) -> SortColumn<DiskColumn> {
    match column_type {
        DiskColumn::Disk => SortColumn::soft(DiskColumn::Disk, Some(0.2)),
        DiskColumn::Mount => SortColumn::soft(DiskColumn::Mount, Some(0.2)),
        DiskColumn::Used => SortColumn::hard(DiskColumn::Used, 8).default_descending(),
        DiskColumn::Free => SortColumn::hard(DiskColumn::Free, 8).default_descending(),
        DiskColumn::Total => SortColumn::hard(DiskColumn::Total, 9).default_descending(),
        DiskColumn::UsedPercent => {
            SortColumn::hard(DiskColumn::UsedPercent, 9).default_descending()
        }
        DiskColumn::FreePercent => {
            SortColumn::hard(DiskColumn::FreePercent, 9).default_descending()
        }
        DiskColumn::IoRead => SortColumn::hard(DiskColumn::IoRead, 10).default_descending(),
        DiskColumn::IoWrite => SortColumn::hard(DiskColumn::IoWrite, 11).default_descending(),
    }
}

const fn default_disk_columns() -> [SortColumn<DiskColumn>; 8] {
    [
        create_column(&DiskColumn::Disk),
        create_column(&DiskColumn::Mount),
        create_column(&DiskColumn::Used),
        create_column(&DiskColumn::Free),
        create_column(&DiskColumn::Total),
        create_column(&DiskColumn::UsedPercent),
        create_column(&DiskColumn::IoRead),
        create_column(&DiskColumn::IoWrite),
    ]
}

impl DiskTableWidget {
    pub fn new(config: &AppConfigFields, palette: &Styles, columns: Option<&[DiskColumn]>) -> Self {
        let props = SortDataTableProps {
            inner: DataTableProps {
                title: Some(" Disks ".into()),
                table_gap: config.table_gap,
                left_to_right: true,
                is_basic: config.use_basic_mode,
                show_table_scroll_position: config.show_table_scroll_position,
                show_current_entry_when_unfocused: false,
            },
            sort_index: 0,
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
