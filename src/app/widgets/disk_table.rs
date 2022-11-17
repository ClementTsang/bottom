use std::{borrow::Cow, cmp::max};

use kstring::KString;
use tui::text::Text;

use crate::{
    app::AppConfigFields,
    canvas::canvas_colours::CanvasColours,
    components::data_table::{
        ColumnHeader, DataTableColumn, DataTableProps, DataTableStyling, DataToCell, SortColumn,
        SortDataTable, SortDataTableProps, SortOrder, SortsRow,
    },
    utils::gen_util::{get_decimal_bytes, sort_partial_fn, truncate_text},
};

#[derive(Clone, Debug)]
pub struct DiskWidgetData {
    pub name: KString,
    pub mount_point: KString,
    pub free_bytes: Option<u64>,
    pub used_bytes: Option<u64>,
    pub total_bytes: Option<u64>,
    pub io_read: KString,
    pub io_write: KString,
}

impl DiskWidgetData {
    pub fn free_space(&self) -> KString {
        if let Some(free_bytes) = self.free_bytes {
            let converted_free_space = get_decimal_bytes(free_bytes);
            format!("{:.*}{}", 0, converted_free_space.0, converted_free_space.1).into()
        } else {
            "N/A".into()
        }
    }

    pub fn total_space(&self) -> KString {
        if let Some(total_bytes) = self.total_bytes {
            let converted_total_space = get_decimal_bytes(total_bytes);
            format!(
                "{:.*}{}",
                0, converted_total_space.0, converted_total_space.1
            )
            .into()
        } else {
            "N/A".into()
        }
    }

    pub fn usage(&self) -> KString {
        if let (Some(used_bytes), Some(total_bytes)) = (self.used_bytes, self.total_bytes) {
            format!("{:.0}%", used_bytes as f64 / total_bytes as f64 * 100_f64).into()
        } else {
            "N/A".into()
        }
    }
}

pub enum DiskWidgetColumn {
    Disk,
    Mount,
    Used,
    Free,
    Total,
    IoRead,
    IoWrite,
}

impl ColumnHeader for DiskWidgetColumn {
    fn text(&self) -> Cow<'static, str> {
        match self {
            DiskWidgetColumn::Disk => "Disk(d)",
            DiskWidgetColumn::Mount => "Mount(m)",
            DiskWidgetColumn::Used => "Used(u)",
            DiskWidgetColumn::Free => "Free(n)",
            DiskWidgetColumn::Total => "Total(t)",
            DiskWidgetColumn::IoRead => "R/s(r)",
            DiskWidgetColumn::IoWrite => "W/s(w)",
        }
        .into()
    }
}

impl DataToCell<DiskWidgetColumn> for DiskWidgetData {
    fn to_cell<'a>(&'a self, column: &DiskWidgetColumn, calculated_width: u16) -> Option<Text<'a>> {
        let text = match column {
            DiskWidgetColumn::Disk => truncate_text(&self.name, calculated_width),
            DiskWidgetColumn::Mount => truncate_text(&self.mount_point, calculated_width),
            DiskWidgetColumn::Used => truncate_text(&self.usage(), calculated_width),
            DiskWidgetColumn::Free => truncate_text(&self.free_space(), calculated_width),
            DiskWidgetColumn::Total => truncate_text(&self.total_space(), calculated_width),
            DiskWidgetColumn::IoRead => truncate_text(&self.io_read, calculated_width),
            DiskWidgetColumn::IoWrite => truncate_text(&self.io_write, calculated_width),
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
            DiskWidgetColumn::Free => {
                data.sort_by(|a, b| sort_partial_fn(descending)(&a.free_bytes, &b.free_bytes));
            }
            DiskWidgetColumn::Total => {
                data.sort_by(|a, b| sort_partial_fn(descending)(&a.total_bytes, &b.total_bytes));
            }
            DiskWidgetColumn::IoRead => {
                data.sort_by(|a, b| sort_partial_fn(descending)(&a.io_read, &b.io_read));
            }
            DiskWidgetColumn::IoWrite => {
                data.sort_by(|a, b| sort_partial_fn(descending)(&a.io_write, &b.io_write));
            }
        }
    }
}

impl DiskTableWidget {
    pub fn new(config: &AppConfigFields, colours: &CanvasColours) -> Self {
        let columns = [
            SortColumn::soft(DiskWidgetColumn::Disk, Some(0.2)),
            SortColumn::soft(DiskWidgetColumn::Mount, Some(0.2)),
            SortColumn::hard(DiskWidgetColumn::Used, 8).default_descending(),
            SortColumn::hard(DiskWidgetColumn::Free, 8).default_descending(),
            SortColumn::hard(DiskWidgetColumn::Total, 9).default_descending(),
            SortColumn::hard(DiskWidgetColumn::IoRead, 10).default_descending(),
            SortColumn::hard(DiskWidgetColumn::IoWrite, 11).default_descending(),
        ];

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

    pub fn ingest_data(&mut self, data: &[DiskWidgetData]) {
        let mut data = data.to_vec();
        if let Some(column) = self.table.columns.get(self.table.sort_index()) {
            column.sort_by(&mut data, self.table.order());
        }
        self.table.set_data(data);
    }
}
