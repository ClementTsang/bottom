use crate::components::text_table::{
    CellContent, TableComponentColumn, TableComponentState, WidthBounds,
};

pub struct DiskWidgetState {
    pub table_state: TableComponentState,
}

impl Default for DiskWidgetState {
    fn default() -> Self {
        const DISK_HEADERS: [&str; 7] = ["Disk", "Mount", "Used", "Free", "Total", "R/s", "W/s"];
        const WIDTHS: [WidthBounds; DISK_HEADERS.len()] = [
            WidthBounds::soft_from_str(DISK_HEADERS[0], Some(0.2)),
            WidthBounds::soft_from_str(DISK_HEADERS[1], Some(0.2)),
            WidthBounds::Hard(4),
            WidthBounds::Hard(6),
            WidthBounds::Hard(6),
            WidthBounds::Hard(7),
            WidthBounds::Hard(7),
        ];

        DiskWidgetState {
            table_state: TableComponentState::new(
                DISK_HEADERS
                    .iter()
                    .zip(WIDTHS)
                    .map(|(header, width)| {
                        TableComponentColumn::new_custom(CellContent::new(*header, None), width)
                    })
                    .collect(),
            ),
        }
    }
}
