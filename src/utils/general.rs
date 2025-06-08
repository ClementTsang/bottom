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
}
