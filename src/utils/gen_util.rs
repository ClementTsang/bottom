use std::cmp::Ordering;

pub fn float_min(a: f32, b: f32) -> f32 {
	match a.partial_cmp(&b) {
		Some(x) => match x {
			Ordering::Greater => b,
			Ordering::Less => a,
			Ordering::Equal => a,
		},
		None => a,
	}
}

pub fn float_max(a: f32, b: f32) -> f32 {
	match a.partial_cmp(&b) {
		Some(x) => match x {
			Ordering::Greater => a,
			Ordering::Less => b,
			Ordering::Equal => a,
		},
		None => a,
	}
}

/// Returns a tuple containing the value and the unit.  In units of 1024.
/// This only supports up to a tebibyte.
pub fn get_exact_byte_values(bytes: u64, spacing: bool) -> (f64, String) {
	match bytes {
		b if b < 1024 => (
			bytes as f64,
			if spacing {
				"  B".to_string()
			} else {
				"B".to_string()
			},
		),
		b if b < 1_048_576 => (bytes as f64 / 1024.0, "KiB".to_string()),
		b if b < 1_073_741_824 => (bytes as f64 / 1_048_576.0, "MiB".to_string()),
		b if b < 1_099_511_627_776 => (bytes as f64 / 1_073_741_824.0, "GiB".to_string()),
		_ => (bytes as f64 / 1_099_511_627_776.0, "TiB".to_string()),
	}
}

/// Returns a tuple containing the value and the unit.  In units of 1000.
/// This only supports up to a terabyte.  Note the "byte" unit will have a space appended to match the others.
pub fn get_simple_byte_values(bytes: u64, spacing: bool) -> (f64, String) {
	match bytes {
		b if b < 1000 => (
			bytes as f64,
			if spacing {
				" B".to_string()
			} else {
				"B".to_string()
			},
		),
		b if b < 1_000_000 => (bytes as f64 / 1000.0, "KB".to_string()),
		b if b < 1_000_000_000 => (bytes as f64 / 1_000_000.0, "MB".to_string()),
		b if b < 1_000_000_000_000 => (bytes as f64 / 1_000_000_000.0, "GB".to_string()),
		_ => (bytes as f64 / 1_000_000_000_000.0, "TB".to_string()),
	}
}

/// Gotta get partial ordering?  No problem, here's something to deal with it~
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
