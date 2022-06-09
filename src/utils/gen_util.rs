use std::{borrow::Cow, cmp::Ordering};

use concat_string::concat_string;
use tui::text::Text;
use unicode_segmentation::UnicodeSegmentation;

pub const KILO_LIMIT: u64 = 1000;
pub const MEGA_LIMIT: u64 = 1_000_000;
pub const GIGA_LIMIT: u64 = 1_000_000_000;
pub const TERA_LIMIT: u64 = 1_000_000_000_000;
pub const KIBI_LIMIT: u64 = 1024;
pub const MEBI_LIMIT: u64 = 1_048_576;
pub const GIBI_LIMIT: u64 = 1_073_741_824;
pub const TEBI_LIMIT: u64 = 1_099_511_627_776;

pub const KILO_LIMIT_F64: f64 = 1000.0;
pub const MEGA_LIMIT_F64: f64 = 1_000_000.0;
pub const GIGA_LIMIT_F64: f64 = 1_000_000_000.0;
pub const TERA_LIMIT_F64: f64 = 1_000_000_000_000.0;
pub const KIBI_LIMIT_F64: f64 = 1024.0;
pub const MEBI_LIMIT_F64: f64 = 1_048_576.0;
pub const GIBI_LIMIT_F64: f64 = 1_073_741_824.0;
pub const TEBI_LIMIT_F64: f64 = 1_099_511_627_776.0;

pub const LOG_KILO_LIMIT: f64 = 3.0;
pub const LOG_MEGA_LIMIT: f64 = 6.0;
pub const LOG_GIGA_LIMIT: f64 = 9.0;
pub const LOG_TERA_LIMIT: f64 = 12.0;
pub const LOG_PETA_LIMIT: f64 = 15.0;

pub const LOG_KIBI_LIMIT: f64 = 10.0;
pub const LOG_MEBI_LIMIT: f64 = 20.0;
pub const LOG_GIBI_LIMIT: f64 = 30.0;
pub const LOG_TEBI_LIMIT: f64 = 40.0;
pub const LOG_PEBI_LIMIT: f64 = 50.0;

pub const LOG_KILO_LIMIT_U32: u32 = 3;
pub const LOG_MEGA_LIMIT_U32: u32 = 6;
pub const LOG_GIGA_LIMIT_U32: u32 = 9;
pub const LOG_TERA_LIMIT_U32: u32 = 12;

pub const LOG_KIBI_LIMIT_U32: u32 = 10;
pub const LOG_MEBI_LIMIT_U32: u32 = 20;
pub const LOG_GIBI_LIMIT_U32: u32 = 30;
pub const LOG_TEBI_LIMIT_U32: u32 = 40;

/// Returns a tuple containing the value and the unit in bytes.  In units of 1024.
/// This only supports up to a tebi.  Note the "single" unit will have a space appended to match the others if
/// `spacing` is true.
pub fn get_binary_bytes(bytes: u64) -> (f64, String) {
    match bytes {
        b if b < KIBI_LIMIT => (bytes as f64, "B".to_string()),
        b if b < MEBI_LIMIT => (bytes as f64 / 1024.0, "KiB".to_string()),
        b if b < GIBI_LIMIT => (bytes as f64 / 1_048_576.0, "MiB".to_string()),
        b if b < TERA_LIMIT => (bytes as f64 / 1_073_741_824.0, "GiB".to_string()),
        _ => (bytes as f64 / 1_099_511_627_776.0, "TiB".to_string()),
    }
}

/// Returns a tuple containing the value and the unit in bytes.  In units of 1000.
/// This only supports up to a tera.  Note the "single" unit will have a space appended to match the others if
/// `spacing` is true.
pub fn get_decimal_bytes(bytes: u64) -> (f64, String) {
    match bytes {
        b if b < KILO_LIMIT => (bytes as f64, "B".to_string()),
        b if b < MEGA_LIMIT => (bytes as f64 / 1000.0, "KB".to_string()),
        b if b < GIGA_LIMIT => (bytes as f64 / 1_000_000.0, "MB".to_string()),
        b if b < TERA_LIMIT => (bytes as f64 / 1_000_000_000.0, "GB".to_string()),
        _ => (bytes as f64 / 1_000_000_000_000.0, "TB".to_string()),
    }
}

/// Returns a tuple containing the value and the unit.  In units of 1024.
/// This only supports up to a tebi.  Note the "single" unit will have a space appended to match the others if
/// `spacing` is true.
pub fn get_binary_prefix(quantity: u64, unit: &str) -> (f64, String) {
    match quantity {
        b if b < KIBI_LIMIT => (quantity as f64, unit.to_string()),
        b if b < MEBI_LIMIT => (quantity as f64 / 1024.0, format!("Ki{}", unit)),
        b if b < GIBI_LIMIT => (quantity as f64 / 1_048_576.0, format!("Mi{}", unit)),
        b if b < TERA_LIMIT => (quantity as f64 / 1_073_741_824.0, format!("Gi{}", unit)),
        _ => (quantity as f64 / 1_099_511_627_776.0, format!("Ti{}", unit)),
    }
}

/// Returns a tuple containing the value and the unit.  In units of 1000.
/// This only supports up to a tera.  Note the "single" unit will have a space appended to match the others if
/// `spacing` is true.
pub fn get_decimal_prefix(quantity: u64, unit: &str) -> (f64, String) {
    match quantity {
        b if b < KILO_LIMIT => (quantity as f64, unit.to_string()),
        b if b < MEGA_LIMIT => (quantity as f64 / 1000.0, format!("K{}", unit)),
        b if b < GIGA_LIMIT => (quantity as f64 / 1_000_000.0, format!("M{}", unit)),
        b if b < TERA_LIMIT => (quantity as f64 / 1_000_000_000.0, format!("G{}", unit)),
        _ => (quantity as f64 / 1_000_000_000_000.0, format!("T{}", unit)),
    }
}

/// Truncates text if it is too long, and adds an ellipsis at the end if needed.
pub fn truncate_text<'a>(content: Cow<'static, str>, width: usize) -> Text<'a> {
    let graphemes: Vec<&str> = UnicodeSegmentation::graphemes(content.as_ref(), true).collect();
    if graphemes.len() > width && width > 0 {
        // Truncate with ellipsis
        let first_n = graphemes[..(width - 1)].concat();
        Text::raw(concat_string!(first_n, "…"))
    } else {
        Text::raw(content)
    }
}

#[inline]
pub fn sort_partial_fn<T: std::cmp::PartialOrd>(is_reverse: bool) -> fn(T, T) -> Ordering {
    if is_reverse {
        partial_ordering_rev
    } else {
        partial_ordering
    }
}

/// Returns an [`Ordering`] between two [`PartialOrd`]s.
#[inline]
pub fn partial_ordering<T: std::cmp::PartialOrd>(a: T, b: T) -> Ordering {
    // TODO: Switch to `total_cmp` on 1.62
    a.partial_cmp(&b).unwrap_or(Ordering::Equal)
}

/// Returns a reversed [`Ordering`] between two [`PartialOrd`]s.
///
/// This is simply a wrapper function around [`partial_ordering`] that reverses
/// the result.
#[inline]
pub fn partial_ordering_rev<T: std::cmp::PartialOrd>(a: T, b: T) -> Ordering {
    partial_ordering(a, b).reverse()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_sort_partial_fn() {
        let mut x = vec![9, 5, 20, 15, 10, 5];
        let mut y = vec![1.0, 15.0, -1.0, -100.0, -100.1, 16.15, -100.0];

        x.sort_by(|a, b| sort_partial_fn(false)(a, b));
        assert_eq!(x, vec![5, 5, 9, 10, 15, 20]);

        x.sort_by(|a, b| sort_partial_fn(true)(a, b));
        assert_eq!(x, vec![20, 15, 10, 9, 5, 5]);

        y.sort_by(|a, b| sort_partial_fn(false)(a, b));
        assert_eq!(y, vec![-100.1, -100.0, -100.0, -1.0, 1.0, 15.0, 16.15]);

        y.sort_by(|a, b| sort_partial_fn(true)(a, b));
        assert_eq!(y, vec![16.15, 15.0, 1.0, -1.0, -100.0, -100.0, -100.1]);
    }
}
