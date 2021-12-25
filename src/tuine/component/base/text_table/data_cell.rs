use std::{borrow::Cow, fmt::Display};

use enum_dispatch::enum_dispatch;
use tui::widgets::Cell;

#[enum_dispatch]
pub trait Numeric {}
impl Numeric for f64 {}
impl Numeric for f32 {}
impl Numeric for i64 {}
impl Numeric for i32 {}
impl Numeric for i16 {}
impl Numeric for i8 {}
impl Numeric for isize {}
impl Numeric for u64 {}
impl Numeric for u32 {}
impl Numeric for u16 {}
impl Numeric for u8 {}
impl Numeric for usize {}

#[allow(non_camel_case_types)]
#[enum_dispatch(Numeric)]
pub enum Number {
    f64,
    f32,
    i64,
    i32,
    i16,
    i8,
    isize,
    u64,
    u32,
    u16,
    u8,
    usize,
}

impl Display for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Number::f64(val) => write!(f, "{}", val),
            Number::f32(val) => write!(f, "{}", val),
            Number::i64(val) => write!(f, "{}", val),
            Number::i32(val) => write!(f, "{}", val),
            Number::i16(val) => write!(f, "{}", val),
            Number::i8(val) => write!(f, "{}", val),
            Number::isize(val) => write!(f, "{}", val),
            Number::u64(val) => write!(f, "{}", val),
            Number::u32(val) => write!(f, "{}", val),
            Number::u16(val) => write!(f, "{}", val),
            Number::u8(val) => write!(f, "{}", val),
            Number::usize(val) => write!(f, "{}", val),
        }
    }
}

pub enum DataCell {
    NumberCell(Number),
    String(Cow<'static, str>),
}

impl DataCell {}

impl Display for DataCell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataCell::NumberCell(n) => n.fmt(f),
            DataCell::String(d) => d.fmt(f),
        }
    }
}

impl From<DataCell> for Cell<'_> {
    fn from(data_cell: DataCell) -> Self {
        Cell::from(data_cell.to_string())
    }
}

impl From<Number> for DataCell {
    fn from(num: Number) -> Self {
        DataCell::NumberCell(num)
    }
}

impl From<String> for DataCell {
    fn from(s: String) -> Self {
        DataCell::String(s.into())
    }
}

impl From<&'static str> for DataCell {
    fn from(s: &'static str) -> Self {
        DataCell::String(s.into())
    }
}
