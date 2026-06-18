use std::cmp::Ordering;

#[inline]
pub(crate) const fn sort_partial_fn<T: PartialOrd>(is_descending: bool) -> fn(T, T) -> Ordering {
    if is_descending {
        partial_ordering_desc
    } else {
        partial_ordering
    }
}

/// Returns an [`Ordering`] between two [`PartialOrd`]s.
#[inline]
pub(crate) fn partial_ordering<T: PartialOrd>(a: T, b: T) -> Ordering {
    a.partial_cmp(&b).unwrap_or(Ordering::Equal)
}

/// Returns a reversed [`Ordering`] between two [`PartialOrd`]s.
///
/// This is simply a wrapper function around [`partial_ordering`] that reverses
/// the result.
#[inline]
pub(crate) fn partial_ordering_desc<T: PartialOrd>(a: T, b: T) -> Ordering {
    partial_ordering(a, b).reverse()
}

/// Returns the digit run starting at `i` with its leading zeroes stripped,
/// alongside the full run's length. `bytes[i]` is assumed to be an ASCII digit.
fn digit_run(bytes: &[u8], i: usize) -> (&[u8], usize) {
    let start = i;
    let mut end = i;
    while end < bytes.len() && bytes[end].is_ascii_digit() {
        end += 1;
    }

    let run = &bytes[start..end];
    let significant = &run[run.iter().take_while(|&&c| c == b'0').count()..];
    (significant, run.len())
}

/// Compares two strings using "natural" ordering, where consecutive runs of
/// ASCII digits are compared by their numeric value rather than character by
/// character. For example, `"core 2"` is ordered before `"core 10"`, unlike the
/// default lexicographic ordering which would place `"core 10"` first.
///
/// Non-digit bytes are compared by their usual ordering, which for UTF-8 matches
/// `char` (Unicode scalar value) ordering. When two numeric runs share the same
/// value, the one with more leading zeroes is ordered afterwards so that the
/// comparison remains a total order.
///
/// This walks the strings as byte slices and allocates nothing, since ASCII
/// digits are always single bytes and never appear inside a multi-byte UTF-8
/// sequence.
pub(crate) fn natural_cmp(a: &str, b: &str) -> Ordering {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    let (mut ai, mut bi) = (0, 0);

    while ai < a.len() && bi < b.len() {
        let (ac, bc) = (a[ai], b[bi]);

        if ac.is_ascii_digit() && bc.is_ascii_digit() {
            let (a_sig, a_len) = digit_run(a, ai);
            let (b_sig, b_len) = digit_run(b, bi);

            // A longer run of significant digits is a larger number; for equal
            // lengths a byte-wise comparison matches the numeric one.
            let by_value = a_sig.len().cmp(&b_sig.len()).then_with(|| a_sig.cmp(b_sig));
            if by_value != Ordering::Equal {
                return by_value;
            }

            // Same numeric value; fewer leading zeroes sorts first.
            let by_zeroes = a_len.cmp(&b_len);
            if by_zeroes != Ordering::Equal {
                return by_zeroes;
            }

            ai += a_len;
            bi += b_len;
        } else {
            let by_byte = ac.cmp(&bc);
            if by_byte != Ordering::Equal {
                return by_byte;
            }
            ai += 1;
            bi += 1;
        }
    }

    // Whichever string still has bytes left is the longer, and thus greater, one;
    // a prefix sorts before the string that extends it.
    (a.len() - ai).cmp(&(b.len() - bi))
}

/// Compares two strings for sorting, optionally using [`natural_cmp`] instead of
/// the default lexicographic ordering, and reverses the result when
/// `descending` is set.
#[inline]
pub(crate) fn sort_str_fn(a: &str, b: &str, descending: bool, natural: bool) -> Ordering {
    let ordering = if natural { natural_cmp(a, b) } else { a.cmp(b) };

    if descending {
        ordering.reverse()
    } else {
        ordering
    }
}

/// A trait for additional clamping functions on numeric types.
pub(crate) trait ClampExt {
    /// Restrict a value by a lower bound. If the current value is _lower_ than
    /// `lower_bound`, it will be set to `_lower_bound`.
    #[cfg_attr(not(test), expect(dead_code))]
    fn clamp_lower(&self, lower_bound: Self) -> Self;

    /// Restrict a value by an upper bound. If the current value is _greater_
    /// than `upper_bound`, it will be set to `upper_bound`.
    fn clamp_upper(&self, upper_bound: Self) -> Self;
}

macro_rules! clamp_num_impl {
    ( $($NumType:ty),+ $(,)? ) => {
        $(
            impl ClampExt for $NumType {
                fn clamp_lower(&self, lower_bound: Self) -> Self {
                    if *self < lower_bound {
                        lower_bound
                    } else {
                        *self
                    }
                }

                fn clamp_upper(&self, upper_bound: Self) -> Self {
                    if *self > upper_bound {
                        upper_bound
                    } else {
                        *self
                    }
                }
            }
        )*
    };
}

clamp_num_impl!(u8, u16, u32, u64, usize);

/// Checked log2.
pub(crate) fn saturating_log2(value: f64) -> f64 {
    if value > 0.0 { value.log2() } else { 0.0 }
}

/// Checked log10.
pub(crate) fn saturating_log10(value: f64) -> f64 {
    if value > 0.0 { value.log10() } else { 0.0 }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_clamp_upper() {
        let val: usize = 100;
        assert_eq!(val.clamp_upper(150), 100);

        let val: usize = 100;
        assert_eq!(val.clamp_upper(100), 100);

        let val: usize = 100;
        assert_eq!(val.clamp_upper(50), 50);
    }

    #[test]
    fn test_clamp_lower() {
        let val: usize = 100;
        assert_eq!(val.clamp_lower(150), 150);

        let val: usize = 100;
        assert_eq!(val.clamp_lower(100), 100);

        let val: usize = 100;
        assert_eq!(val.clamp_lower(50), 100);
    }

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

    #[test]
    fn test_natural_cmp() {
        // Embedded numbers are compared by value, not lexicographically.
        assert_eq!(natural_cmp("core 2", "core 10"), Ordering::Less);
        assert_eq!(natural_cmp("core 10", "core 2"), Ordering::Greater);
        assert_eq!(natural_cmp("core 2", "core 2"), Ordering::Equal);

        // Leading zeroes do not change the numeric value, only the tiebreak.
        assert_eq!(natural_cmp("core 02", "core 2"), Ordering::Greater);
        assert_eq!(natural_cmp("v0", "v00"), Ordering::Less);

        // Falls back to character ordering outside of digit runs.
        assert_eq!(natural_cmp("abc", "abd"), Ordering::Less);
        assert_eq!(natural_cmp("abc", "abc"), Ordering::Equal);

        // Shorter string is "less" when it is a prefix of the other.
        assert_eq!(natural_cmp("core", "core 1"), Ordering::Less);
        assert_eq!(natural_cmp("", "a"), Ordering::Less);

        // Multiple numeric runs.
        assert_eq!(natural_cmp("x2y9", "x2y10"), Ordering::Less);
        assert_eq!(natural_cmp("x10y1", "x2y1"), Ordering::Greater);
    }

    #[test]
    fn test_natural_sort() {
        let mut entries = vec!["core 10", "core 1", "core 2"];
        entries.sort_by(|a, b| natural_cmp(a, b));
        assert_eq!(entries, vec!["core 1", "core 2", "core 10"]);

        // The default lexicographic ordering interleaves by character instead.
        let mut lexicographic = vec!["core 10", "core 1", "core 2"];
        lexicographic.sort();
        assert_eq!(lexicographic, vec!["core 1", "core 10", "core 2"]);
    }

    #[test]
    fn test_sort_str_fn() {
        // Lexicographic when natural is off.
        assert_eq!(
            sort_str_fn("core 10", "core 2", false, false),
            Ordering::Less
        );
        // Natural when on.
        assert_eq!(
            sort_str_fn("core 10", "core 2", false, true),
            Ordering::Greater
        );
        // Descending flips the result.
        assert_eq!(sort_str_fn("core 10", "core 2", true, true), Ordering::Less);
    }
}
