#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub enum DataUnit {
    Byte,
    #[default]
    Bit,
}

pub const KILO_LIMIT: u64 = 1000;
pub const MEGA_LIMIT: u64 = 1_000_000;
pub const GIGA_LIMIT: u64 = 1_000_000_000;
pub const TERA_LIMIT: u64 = 1_000_000_000_000;
pub const KIBI_LIMIT: u64 = 1024;
pub const MEBI_LIMIT: u64 = 1024 * 1024;
pub const GIBI_LIMIT: u64 = 1024 * 1024 * 1024;
pub const TEBI_LIMIT: u64 = 1024 * 1024 * 1024 * 1024;

pub const KILO_LIMIT_F64: f64 = 1000.0;
pub const MEGA_LIMIT_F64: f64 = 1_000_000.0;
pub const GIGA_LIMIT_F64: f64 = 1_000_000_000.0;
pub const TERA_LIMIT_F64: f64 = 1_000_000_000_000.0;
pub const KIBI_LIMIT_F64: f64 = 1024.0;
pub const MEBI_LIMIT_F64: f64 = 1024.0 * 1024.0;
pub const GIBI_LIMIT_F64: f64 = 1024.0 * 1024.0 * 1024.0;
pub const TEBI_LIMIT_F64: f64 = 1024.0 * 1024.0 * 1024.0 * 1024.0;

pub const LOG_MEGA_LIMIT: f64 = 6.0;
pub const LOG_GIGA_LIMIT: f64 = 9.0;
pub const LOG_TERA_LIMIT: f64 = 12.0;
pub const LOG_PETA_LIMIT: f64 = 15.0;

pub const LOG_MEBI_LIMIT: f64 = 20.0;
pub const LOG_GIBI_LIMIT: f64 = 30.0;
pub const LOG_TEBI_LIMIT: f64 = 40.0;
pub const LOG_PEBI_LIMIT: f64 = 50.0;

/// Returns a tuple containing the value and the unit in bytes. In units of
/// 1024. This only supports up to a tebi.  Note the "single" unit will have a
/// space appended to match the others if `spacing` is true.
#[inline]
pub fn get_binary_bytes(bytes: u64) -> (f64, &'static str) {
    match bytes {
        b if b < KIBI_LIMIT => (bytes as f64, "B"),
        b if b < MEBI_LIMIT => (bytes as f64 / KIBI_LIMIT_F64, "KiB"),
        b if b < GIBI_LIMIT => (bytes as f64 / MEBI_LIMIT_F64, "MiB"),
        b if b < TEBI_LIMIT => (bytes as f64 / GIBI_LIMIT_F64, "GiB"),
        _ => (bytes as f64 / TEBI_LIMIT_F64, "TiB"),
    }
}

/// Returns a tuple containing the value and the unit in bytes. In units of
/// 1000. This only supports up to a tera.  Note the "single" unit will have a
/// space appended to match the others if `spacing` is true.
#[inline]
pub fn get_decimal_bytes(bytes: u64) -> (f64, &'static str) {
    match bytes {
        b if b < KILO_LIMIT => (bytes as f64, "B"),
        b if b < MEGA_LIMIT => (bytes as f64 / KILO_LIMIT_F64, "KB"),
        b if b < GIGA_LIMIT => (bytes as f64 / MEGA_LIMIT_F64, "MB"),
        b if b < TERA_LIMIT => (bytes as f64 / GIGA_LIMIT_F64, "GB"),
        _ => (bytes as f64 / TERA_LIMIT_F64, "TB"),
    }
}

/// Given a value in _bits_, turn a tuple containing the value and a unit.
#[inline]
pub fn convert_bits(bits: u64, base_two: bool) -> (f64, &'static str) {
    let bytes = bits / 8;

    if base_two {
        get_binary_bytes(bytes)
    } else {
        get_decimal_bytes(bytes)
    }
}

/// Return a tuple containing the value and a unit string to be used as a prefix.
#[inline]
pub fn get_unit_prefix(value: u64, base_two: bool) -> (f64, &'static str) {
    let float_value = value as f64;

    if base_two {
        match value {
            b if b < KIBI_LIMIT => (float_value, ""),
            b if b < MEBI_LIMIT => (float_value / KIBI_LIMIT_F64, "Ki"),
            b if b < GIBI_LIMIT => (float_value / MEBI_LIMIT_F64, "Mi"),
            b if b < TEBI_LIMIT => (float_value / GIBI_LIMIT_F64, "Gi"),
            _ => (float_value / TEBI_LIMIT_F64, "Ti"),
        }
    } else {
        match value {
            b if b < KILO_LIMIT => (float_value, ""),
            b if b < MEGA_LIMIT => (float_value / KILO_LIMIT_F64, "K"),
            b if b < GIGA_LIMIT => (float_value / MEGA_LIMIT_F64, "M"),
            b if b < TERA_LIMIT => (float_value / GIGA_LIMIT_F64, "G"),
            _ => (float_value / TERA_LIMIT_F64, "T"),
        }
    }
}
