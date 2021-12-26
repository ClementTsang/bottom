#[derive(Clone, Copy, PartialEq)]
pub enum SortType {
    Unsortable,
    Ascending(usize),
    Descending(usize),
}

impl Default for SortType {
    fn default() -> Self {
        Self::Unsortable
    }
}
