#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub enum DataUnit {
    Byte,
    #[default]
    Bit,
}
