use std::cmp::max;

use kstring::KString;
use tui::widgets::Row;

use crate::{
    app::AppConfigFields,
    canvas::canvas_colours::CanvasColours,
    components::data_table::{Column, DataTable, DataTableProps, DataTableStyling, ToDataRow},
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
    fn to_data_row<'a>(&self, widths: &[u16]) -> Row<'a> {
        Row::new(vec![
            truncate_text(self.name.clone().into_cow_str(), widths[0].into()),
            truncate_text(self.mount_point.clone().into_cow_str(), widths[1].into()),
            truncate_text(self.free_space().into_cow_str(), widths[2].into()),
            truncate_text(self.total_space().into_cow_str(), widths[3].into()),
            truncate_text(self.usage().into_cow_str(), widths[4].into()),
            truncate_text(self.io_read.clone().into_cow_str(), widths[5].into()),
            truncate_text(self.io_write.clone().into_cow_str(), widths[6].into()),
        ])
    }

    fn column_widths(data: &[DiskWidgetData]) -> Vec<u16>
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
    pub table: DataTable<DiskWidgetData>,
}

impl DiskTableWidget {
    pub fn new(config: &AppConfigFields, colours: &CanvasColours) -> Self {
        const COLUMNS: [Column<&str>; 7] = [
            Column::soft("Disk", Some(0.2)),
            Column::soft("Mount", Some(0.2)),
            Column::hard("Used", 4),
            Column::hard("Free", 6),
            Column::hard("Total", 6),
            Column::hard("R/s", 7),
            Column::hard("W/s", 7),
        ];

        let props = DataTableProps {
            title: Some(" Disks ".into()),
            table_gap: config.table_gap,
            left_to_right: true,
            is_basic: config.use_basic_mode,
            show_table_scroll_position: config.show_table_scroll_position,
            show_current_entry_when_unfocused: false,
        };

        let styling = DataTableStyling {
            header_style: colours.table_header_style,
            border_style: colours.border_style,
            highlighted_border_style: colours.highlighted_border_style,
            text_style: colours.text_style,
            highlighted_text_style: colours.currently_selected_text_style,
            title_style: colours.widget_title_style,
        };

        Self {
            table: DataTable::new(COLUMNS, props, styling),
        }
    }
}
