#[derive(Debug, Clone)]
pub enum DataUnit {
    Byte,
    Bit,
}

impl Default for DataUnit {
    fn default() -> Self {
        DataUnit::Bit
    }
}
