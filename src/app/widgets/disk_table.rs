use std::{borrow::Cow, cmp::max};

use kstring::KString;
use tui::text::Text;

use crate::{
    app::AppConfigFields,
    canvas::canvas_colours::CanvasColours,
    components::data_table::{
        Column, ColumnHeader, DataTable, DataTableColumn, DataTableProps, DataTableStyling,
        DataToCell,
    },
    utils::gen_util::{get_decimal_bytes, truncate_text},
};

#[derive(Clone)]
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
            DiskWidgetColumn::Disk => "Disk",
            DiskWidgetColumn::Mount => "Mount",
            DiskWidgetColumn::Used => "Used",
            DiskWidgetColumn::Free => "Free",
            DiskWidgetColumn::Total => "Total",
            DiskWidgetColumn::IoRead => "R/s",
            DiskWidgetColumn::IoWrite => "W/s",
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
    pub table: DataTable<DiskWidgetData, DiskWidgetColumn>,
}

impl DiskTableWidget {
    pub fn new(config: &AppConfigFields, colours: &CanvasColours) -> Self {
        const COLUMNS: [Column<DiskWidgetColumn>; 7] = [
            Column::soft(DiskWidgetColumn::Disk, Some(0.2)),
            Column::soft(DiskWidgetColumn::Mount, Some(0.2)),
            Column::hard(DiskWidgetColumn::Used, 4),
            Column::hard(DiskWidgetColumn::Free, 6),
            Column::hard(DiskWidgetColumn::Total, 6),
            Column::hard(DiskWidgetColumn::IoRead, 7),
            Column::hard(DiskWidgetColumn::IoWrite, 7),
        ];

        let props = DataTableProps {
            title: Some(" Disks ".into()),
            table_gap: config.table_gap,
            left_to_right: true,
            is_basic: config.use_basic_mode,
            show_table_scroll_position: config.show_table_scroll_position,
            show_current_entry_when_unfocused: false,
        };

        let styling = DataTableStyling::from_colours(colours);

        Self {
            table: DataTable::new(COLUMNS, props, styling),
        }
    }

    pub fn ingest_data(&mut self, data: &[DiskWidgetData]) {
        self.table.set_data(data.to_vec());
    }
}
