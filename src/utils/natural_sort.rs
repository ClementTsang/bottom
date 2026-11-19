//! Natural string sorting for bottom. Based on <https://github.com/lifthrasiir/rust-natord>.

use core::array;
use std::cmp::Ordering;

/// A single token.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Token<'s> {
    Text(&'s str),
    Number(&'s str),
}

/// Compares two strings using natural ordering. Note:
///
/// - Numeric substrings are compared by numeric value.
/// - Non-numeric substrings are compared lexicographically.
/// - Shorter strings will come first.
#[inline]
pub fn natural_str_cmp(left: &str, right: &str) -> Ordering {
    // We have a max amount of tokens; 16 was chosen as a reasonable upper limit for typical identifiers.
    const MAX_TOKENS: usize = 16;

    // Tokenize both strings into fixed-size stack buffers.
    let mut l_tokens: [Token<'_>; MAX_TOKENS] = array::from_fn(|_| Token::Text(""));
    let mut r_tokens: [Token<'_>; MAX_TOKENS] = array::from_fn(|_| Token::Text(""));

    let l_count = tokenize(left, &mut l_tokens);
    let r_count = tokenize(right, &mut r_tokens);

    // Compare tokens one-by-one.
    for i in 0..l_count.min(r_count) {
        match natural_cmp_token(l_tokens[i], r_tokens[i]) {
            Ordering::Equal => continue,
            ord => return ord,
        }
    }

    // All compared tokens are equal; shorter token list comes first.
    l_count.cmp(&r_count)
}

/// Tokenize a string into a fixed-size buffer, returning the count written.
fn tokenize<'a>(s: &'a str, tokens: &mut [Token<'a>]) -> usize {
    let mut count: usize = 0;
    let bytes = s.as_bytes();

    if bytes.is_empty() {
        return 0;
    }

    let mut start: usize = 0;
    let mut is_number = bytes[0].is_ascii_digit();

    let mut i = 1usize;
    while i < bytes.len() {
        if bytes[i].is_ascii_digit() != is_number {
            if count < tokens.len() {
                tokens[count] = if is_number {
                    Token::Number(&s[start..i])
                } else {
                    Token::Text(&s[start..i])
                };
                count += 1;
            }
            start = i;
            is_number = bytes[i].is_ascii_digit();
        }
        i += 1;
    }

    // Emit final segment if buffer has room.
    if count < tokens.len() {
        tokens[count] = if is_number {
            Token::Number(&s[start..])
        } else {
            Token::Text(&s[start..])
        };
        count += 1;
    }

    count
}

/// Compare two tokens at the same position.
#[inline]
fn natural_cmp_token(left: Token<'_>, right: Token<'_>) -> Ordering {
    match (left, right) {
        (Token::Text(l), Token::Text(r)) => l.cmp(r),
        (Token::Number(l), Token::Number(r)) => {
            let l_bytes = l.as_bytes();
            let r_bytes = r.as_bytes();
            match l_bytes.len().cmp(&r_bytes.len()) {
                Ordering::Equal => l_bytes.cmp(r_bytes),
                ord => ord,
            }
        }
        // Text sorts before number if the tokens are not the same type.
        (Token::Text(_), Token::Number(_)) => Ordering::Less,
        (Token::Number(_), Token::Text(_)) => Ordering::Greater,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::Ordering::{Equal, Greater, Less};

    /// Verify that the given list of strings is in natural order.
    fn check_natural_order(strs: &[&str]) {
        for (i, &a) in strs.iter().enumerate() {
            for (j, &b) in strs.iter().enumerate() {
                let actual = natural_str_cmp(a, b);
                let expected = i.cmp(&j);

                assert_eq!(
                    actual, expected,
                    "`{a}` and `{b}` gives {actual:?}, but expected {expected:?}"
                );
            }
        }
    }

    #[test]
    fn test_basic_numeric() {
        check_natural_order(&["cpu1", "cpu2", "cpu10", "cpu11"]);
    }

    #[test]
    fn test_disk_names() {
        check_natural_order(&["sda", "sda1", "sda2", "sda10", "sda20", "sdb"]);
    }

    #[test]
    fn test_multiple_segments() {
        check_natural_order(&["core-01", "core-02", "core-10", "socket-01", "socket-02"]);
    }

    /// Digit runs are compared by length first, then by value.
    /// "1.01" (2-digit run "01") sorts after "1.1" (1-digit run "1").
    #[test]
    fn test_leading_zeros() {
        check_natural_order(&["1", "1.0", "1.1", "1.01", "2"]);
    }

    /// Test that text comes before numbers at same position.
    #[test]
    fn test_mixed_text_and_number() {
        check_natural_order(&["a", "a1", "a01", "a02", "b"]);
    }

    #[test]
    fn test_equal() {
        assert_eq!(natural_str_cmp("hello", "hello"), Equal);
    }

    #[test]
    fn test_empty_strings() {
        assert_eq!(natural_str_cmp("", ""), Equal);
        assert_eq!(natural_str_cmp("", "a"), Less);
        assert_eq!(natural_str_cmp("a", ""), Greater);
        assert_eq!(natural_str_cmp("1", ""), Greater);
        assert_eq!(natural_str_cmp("", "1"), Less);
    }

    #[test]
    fn test_only_numbers() {
        check_natural_order(&["1", "2", "10", "100", "1000"]);
    }

    #[test]
    fn test_only_text() {
        check_natural_order(&["abc", "abca", "abd", "ac"]);
    }

    #[test]
    fn test_case_sensitive() {
        check_natural_order(&["A", "B", "a", "b"]);
    }
}
