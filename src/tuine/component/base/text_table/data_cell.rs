use std::{borrow::Cow, fmt::Display};

use enum_dispatch::enum_dispatch;
use float_ord::FloatOrd;
use tui::widgets::Cell;

#[enum_dispatch]
pub trait DataCellValue {}

impl DataCellValue for FloatOrd<f64> {}
impl DataCellValue for FloatOrd<f32> {}
impl DataCellValue for i64 {}
impl DataCellValue for i32 {}
impl DataCellValue for i16 {}
impl DataCellValue for i8 {}
impl DataCellValue for isize {}
impl DataCellValue for u64 {}
impl DataCellValue for u32 {}
impl DataCellValue for u16 {}
impl DataCellValue for u8 {}
impl DataCellValue for usize {}
impl DataCellValue for Cow<'static, str> {}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[enum_dispatch(DataCellValue)]
pub enum DataCell {
    f64(FloatOrd<f64>),
    f32(FloatOrd<f32>),
    i64(i64),
    i32(i32),
    i16(i16),
    i8(i8),
    isize(isize),
    u64(u64),
    u32(u32),
    u16(u16),
    u8(u8),
    usize(usize),
    Cow(Cow<'static, str>),
}

impl Display for DataCell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataCell::f64(val) => val.0.fmt(f),
            DataCell::f32(val) => val.0.fmt(f),
            DataCell::i64(val) => val.fmt(f),
            DataCell::i32(val) => val.fmt(f),
            DataCell::i16(val) => val.fmt(f),
            DataCell::i8(val) => val.fmt(f),
            DataCell::isize(val) => val.fmt(f),
            DataCell::u64(val) => val.fmt(f),
            DataCell::u32(val) => val.fmt(f),
            DataCell::u16(val) => val.fmt(f),
            DataCell::u8(val) => val.fmt(f),
            DataCell::usize(val) => val.fmt(f),
            DataCell::Cow(val) => val.fmt(f),
        }
    }
}

impl From<DataCell> for Cell<'_> {
    fn from(data_cell: DataCell) -> Self {
        Cell::from(data_cell.to_string())
    }
}

impl From<f64> for DataCell {
    fn from(num: f64) -> Self {
        DataCell::f64(FloatOrd(num))
    }
}

impl From<f32> for DataCell {
    fn from(num: f32) -> Self {
        DataCell::f32(FloatOrd(num))
    }
}

impl From<String> for DataCell {
    fn from(s: String) -> Self {
        DataCell::Cow(Cow::from(s))
    }
}

impl From<&'static str> for DataCell {
    fn from(s: &'static str) -> Self {
        DataCell::Cow(Cow::from(s))
    }
}
