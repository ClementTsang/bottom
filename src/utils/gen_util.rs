use std::cmp::Ordering;

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

/// Gotta get partial ordering?  No problem, here's something to deal with it~
///
/// Note that https://github.com/reem/rust-ordered-float exists, maybe move to it one day?  IDK.
pub fn get_ordering<T: std::cmp::PartialOrd>(
    a_val: T, b_val: T, reverse_order: bool,
) -> std::cmp::Ordering {
    match a_val.partial_cmp(&b_val) {
        Some(x) => match x {
            Ordering::Greater => {
                if reverse_order {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Greater
                }
            }
            Ordering::Less => {
                if reverse_order {
                    std::cmp::Ordering::Greater
                } else {
                    std::cmp::Ordering::Less
                }
            }
            Ordering::Equal => Ordering::Equal,
        },
        None => Ordering::Equal,
    }
}
