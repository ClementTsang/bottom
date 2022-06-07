use crate::components::data_table::ToDataRow;

pub trait SortsRow<Data>
where
    Data: ToDataRow,
{
}
