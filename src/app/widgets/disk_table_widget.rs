use std::cmp::max;

use kstring::KString;
use tui::widgets::Row;

use crate::{
    app::AppConfigFields,
    components::data_table::{DataTable, DataTableColumn, DataTableInner, DataTableProps},
    utils::gen_util::{get_decimal_bytes, truncate_text},
};

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

pub struct DiskWidgetInner;

impl DataTableInner<DiskWidgetData> for DiskWidgetInner {
    fn to_data_row<'a>(&self, data: &'a DiskWidgetData, columns: &[DataTableColumn]) -> Row<'a> {
        Row::new(vec![
            truncate_text(
                data.name.clone().into_cow_str(),
                columns[0].calculated_width.into(),
            ),
            truncate_text(
                data.mount_point.clone().into_cow_str(),
                columns[1].calculated_width.into(),
            ),
            truncate_text(
                data.free_space().into_cow_str(),
                columns[2].calculated_width.into(),
            ),
            truncate_text(
                data.total_space().into_cow_str(),
                columns[3].calculated_width.into(),
            ),
            truncate_text(
                data.usage().into_cow_str(),
                columns[4].calculated_width.into(),
            ),
            truncate_text(
                data.io_read.clone().into_cow_str(),
                columns[5].calculated_width.into(),
            ),
            truncate_text(
                data.io_write.clone().into_cow_str(),
                columns[6].calculated_width.into(),
            ),
        ])
    }

    fn column_widths(&self, data: &[DiskWidgetData]) -> Vec<u16>
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
    pub table: DataTable<DiskWidgetData, DiskWidgetInner>,
}

impl DiskTableWidget {
    pub fn new(config: &AppConfigFields) -> Self {
        const COLUMNS: [DataTableColumn; 7] = [
            DataTableColumn::soft("Disk", Some(0.2)),
            DataTableColumn::soft("Mount", Some(0.2)),
            DataTableColumn::hard("Used", 4),
            DataTableColumn::hard("Free", 6),
            DataTableColumn::hard("Total", 6),
            DataTableColumn::hard("R/s", 7),
            DataTableColumn::hard("W/s", 7),
        ];

        let props = DataTableProps {
            title: Some(" Disks ".into()),
            table_gap: config.table_gap,
            left_to_right: true,
            is_basic: config.use_basic_mode,
            show_table_scroll_position: config.show_table_scroll_position,
            show_current_entry_when_unfocused: false,
        };

        Self {
            table: DataTable::new(COLUMNS, props, DiskWidgetInner {}),
        }
    }
}
