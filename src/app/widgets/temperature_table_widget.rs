use crate::components::text_table::{
    CellContent, TableComponentColumn, TableComponentState, WidthBounds,
};

pub struct TempWidgetState {
    pub table_state: TableComponentState,
}

impl Default for TempWidgetState {
    fn default() -> Self {
        const TEMP_HEADERS: [&str; 2] = ["Sensor", "Temp"];
        const WIDTHS: [WidthBounds; TEMP_HEADERS.len()] = [
            WidthBounds::soft_from_str(TEMP_HEADERS[0], Some(0.8)),
            WidthBounds::soft_from_str(TEMP_HEADERS[1], None),
        ];

        TempWidgetState {
            table_state: TableComponentState::new(
                TEMP_HEADERS
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
