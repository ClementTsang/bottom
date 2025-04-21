//! This mainly concerns converting collected data into things that the canvas
//! can actually handle.

use crate::utils::data_units::*;

/// Returns the most appropriate binary prefix unit type (e.g. kibibyte) and
/// denominator for the given amount of bytes.
///
/// The expected usage is to divide out the given value with the returned
/// denominator in order to be able to use it with the returned binary unit
/// (e.g. divide 3000 bytes by 1024 to have a value in KiB).
#[inline]
pub(crate) fn get_binary_unit_and_denominator(bytes: u64) -> (&'static str, f64) {
    match bytes {
        b if b < KIBI_LIMIT => ("B", 1.0),
        b if b < MEBI_LIMIT => ("KiB", KIBI_LIMIT_F64),
        b if b < GIBI_LIMIT => ("MiB", MEBI_LIMIT_F64),
        b if b < TEBI_LIMIT => ("GiB", GIBI_LIMIT_F64),
        _ => ("TiB", TEBI_LIMIT_F64),
    }
}

/// Returns a string given a value that is converted to the closest SI-variant,
/// per second. If the value is greater than a giga-X, then it will return a
/// decimal place.
#[inline]
pub(crate) fn dec_bytes_per_second_string(value: u64) -> String {
    let converted_values = get_decimal_bytes(value);
    if value >= GIGA_LIMIT {
        format!("{:.1}{}/s", converted_values.0, converted_values.1)
    } else {
        format!("{:.0}{}/s", converted_values.0, converted_values.1)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_dec_bytes_per_second_string() {
        assert_eq!(dec_bytes_per_second_string(0), "0B/s".to_string());
        assert_eq!(dec_bytes_per_second_string(1), "1B/s".to_string());
        assert_eq!(dec_bytes_per_second_string(900), "900B/s".to_string());
        assert_eq!(dec_bytes_per_second_string(999), "999B/s".to_string());
        assert_eq!(dec_bytes_per_second_string(KILO_LIMIT), "1KB/s".to_string());
        assert_eq!(
            dec_bytes_per_second_string(KILO_LIMIT + 1),
            "1KB/s".to_string()
        );
        assert_eq!(dec_bytes_per_second_string(KIBI_LIMIT), "1KB/s".to_string());
        assert_eq!(dec_bytes_per_second_string(MEGA_LIMIT), "1MB/s".to_string());
        assert_eq!(
            dec_bytes_per_second_string(GIGA_LIMIT),
            "1.0GB/s".to_string()
        );
        assert_eq!(
            dec_bytes_per_second_string(2 * GIGA_LIMIT),
            "2.0GB/s".to_string()
        );
        assert_eq!(
            dec_bytes_per_second_string((2.5 * GIGA_LIMIT as f64) as u64),
            "2.5GB/s".to_string()
        );
        assert_eq!(
            dec_bytes_per_second_string((10.34 * TERA_LIMIT as f64) as u64),
            "10.3TB/s".to_string()
        );
        assert_eq!(
            dec_bytes_per_second_string((10.36 * TERA_LIMIT as f64) as u64),
            "10.4TB/s".to_string()
        );
    }
}
