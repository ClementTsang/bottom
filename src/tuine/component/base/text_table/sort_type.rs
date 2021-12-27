#[derive(Clone, Copy, PartialEq)]
pub enum SortType {
    Unsortable,
    Ascending(usize),
    Descending(usize),
}

impl SortType {
    pub(crate) fn prune_length(&mut self, num_columns: usize) {
        match self {
            SortType::Unsortable => {}
            SortType::Ascending(column) | SortType::Descending(column) => {
                if *column >= num_columns {
                    *column = num_columns - 1;
                }
            }
        }
    }
}

impl Default for SortType {
    fn default() -> Self {
        Self::Unsortable
    }
}
