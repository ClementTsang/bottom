#[derive(Debug, Clone, Eq, PartialEq)]
pub enum DataUnit {
    Byte,
    Bit,
}

impl Default for DataUnit {
    fn default() -> Self {
        DataUnit::Bit
    }
}
