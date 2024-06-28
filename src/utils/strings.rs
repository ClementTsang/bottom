use tui::text::Text;
use unicode_ellipsis::truncate_str;

/// Truncates text if it is too long, and adds an ellipsis at the end if needed.
///
/// TODO: Maybe cache results from this function for some cases? e.g. columns
#[inline]
pub fn truncate_to_text<'a, U: Into<usize>>(content: &str, width: U) -> Text<'a> {
    Text::raw(truncate_str(content, width.into()).to_string())
}

/// Checks that the first string is equal to any of the other ones in a ASCII
/// case-insensitive match.
///
/// The generated code is the same as writing:
/// `to_ascii_lowercase(a) == to_ascii_lowercase(b) || to_ascii_lowercase(a) ==
/// to_ascii_lowercase(c)`, but without allocating and copying temporaries.
///
/// # Examples
///
/// ```ignore
/// assert!(multi_eq_ignore_ascii_case!("test", "test"));
/// assert!(multi_eq_ignore_ascii_case!("test", "a" | "b" | "test"));
/// assert!(!multi_eq_ignore_ascii_case!("test", "a" | "b" | "c"));
/// ```
#[macro_export]
macro_rules! multi_eq_ignore_ascii_case {
    ( $lhs:expr, $last:literal ) => {
        $lhs.eq_ignore_ascii_case($last)
    };
    ( $lhs:expr, $head:literal | $($tail:tt)* ) => {
        $lhs.eq_ignore_ascii_case($head) || multi_eq_ignore_ascii_case!($lhs, $($tail)*)
    };
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_multi_eq_ignore_ascii_case() {
        assert!(
            multi_eq_ignore_ascii_case!("test", "test"),
            "single comparison should succeed"
        );
        assert!(
            multi_eq_ignore_ascii_case!("test", "a" | "test"),
            "double comparison should succeed"
        );
        assert!(
            multi_eq_ignore_ascii_case!("test", "a" | "b" | "test"),
            "multi comparison should succeed"
        );

        assert!(
            !multi_eq_ignore_ascii_case!("test", "a"),
            "single non-matching should fail"
        );
        assert!(
            !multi_eq_ignore_ascii_case!("test", "a" | "b"),
            "double non-matching should fail"
        );
        assert!(
            !multi_eq_ignore_ascii_case!("test", "a" | "b" | "c"),
            "multi non-matching should fail"
        );
    }
}
