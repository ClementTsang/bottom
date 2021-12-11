/// Which strategy to use while laying out things.
pub enum Length {
    /// Fill in remaining space. Equivalent to `Length::FlexRatio(1)`.
    Flex,

    /// Fill in remaining space, with the value being a ratio.
    FlexRatio(u16),

    /// Fill in a fixed amount of space.
    Fixed(u16),

    /// Let the child determine how large to make the component.
    Child,
}
