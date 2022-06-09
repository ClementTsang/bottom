use std::cmp::max;

use kstring::KString;
use tui::widgets::Row;

use crate::{
    app::AppConfigFields,
    components::data_table::{DataColumn, DataTable, DataTableProps, ToDataRow},
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

impl ToDataRow for DiskWidgetData {
    fn to_data_row<'a>(&'a self, columns: &[DataColumn]) -> Row<'a> {
        Row::new(vec![
            truncate_text(
                self.name.clone().into_cow_str(),
                columns[0].calculated_width.into(),
            ),
            truncate_text(
                self.mount_point.clone().into_cow_str(),
                columns[1].calculated_width.into(),
            ),
            truncate_text(
                self.free_space().into_cow_str(),
                columns[2].calculated_width.into(),
            ),
            truncate_text(
                self.total_space().into_cow_str(),
                columns[3].calculated_width.into(),
            ),
            truncate_text(
                self.usage().into_cow_str(),
                columns[4].calculated_width.into(),
            ),
            truncate_text(
                self.io_read.clone().into_cow_str(),
                columns[5].calculated_width.into(),
            ),
            truncate_text(
                self.io_write.clone().into_cow_str(),
                columns[6].calculated_width.into(),
            ),
        ])
    }

    fn column_widths(data: &[Self]) -> Vec<u16>
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

pub struct DiskWidgetState {
    pub table: DataTable<DiskWidgetData>,
}

impl DiskWidgetState {
    pub fn new(config: &AppConfigFields) -> Self {
        const COLUMNS: [DataColumn; 7] = [
            DataColumn::soft("Disk", Some(0.2)),
            DataColumn::soft("Mount", Some(0.2)),
            DataColumn::hard("Used", 4),
            DataColumn::hard("Free", 6),
            DataColumn::hard("Total", 6),
            DataColumn::hard("R/s", 7),
            DataColumn::hard("W/s", 7),
        ];

        let props = DataTableProps {
            title: Some(" Disks ".into()),
            table_gap: config.table_gap,
            left_to_right: true,
            is_basic: config.use_basic_mode,
            show_table_scroll_position: config.show_table_scroll_position,
        };

        Self {
            table: DataTable::new(COLUMNS.to_vec(), props),
        }
    }
}
