use std::borrow::Cow;

pub struct TextColumn {
    pub name: Cow<'static, str>,
    pub width_constraint: TextColumnConstraint,
}

pub enum TextColumnConstraint {
    /// Let the column grow to max possible size based on its contents.
    Fill,

    /// The column is exactly as long as specified.
    Length(u16),

    /// The column is exactly as long as specified based on the available area.
    Percentage(u16),

    /// The column will take up as much room as needed, and capped by the given length.
    MaxLength(u16),

    /// The column will take up as much room as needed, and capped by the given length
    /// based on the available area.
    MaxPercentage(u16),
}

impl TextColumn {
    pub fn new<S: Into<Cow<'static, str>>>(name: S) -> Self {
        Self {
            name: name.into(),
            width_constraint: TextColumnConstraint::Fill,
        }
    }

    pub fn width_constraint(mut self, width_constraint: TextColumnConstraint) -> Self {
        self.width_constraint = width_constraint;
        self
    }
}
