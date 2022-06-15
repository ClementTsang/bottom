use crate::components::data_table::ToDataRow;

pub struct ProcWidgetData {}

impl ToDataRow for ProcWidgetData {
    fn to_data_row<'a>(&self, _widths: &[u16]) -> tui::widgets::Row<'a> {
        todo!()
    }

    fn column_widths(_data: &[Self]) -> Vec<u16>
    where
        Self: Sized,
    {
        todo!()
    }
}
