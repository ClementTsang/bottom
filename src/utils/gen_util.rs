use std::cmp::Ordering;

use tui::text::{Span, Spans, Text};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

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
pub fn get_binary_bytes(bytes: u64) -> (f64, &'static str) {
    match bytes {
        b if b < KIBI_LIMIT => (bytes as f64, "B"),
        b if b < MEBI_LIMIT => (bytes as f64 / 1024.0, "KiB"),
        b if b < GIBI_LIMIT => (bytes as f64 / 1_048_576.0, "MiB"),
        b if b < TERA_LIMIT => (bytes as f64 / 1_073_741_824.0, "GiB"),
        _ => (bytes as f64 / 1_099_511_627_776.0, "TiB"),
    }
}

/// Returns a tuple containing the value and the unit in bytes.  In units of 1000.
/// This only supports up to a tera.  Note the "single" unit will have a space appended to match the others if
/// `spacing` is true.
pub fn get_decimal_bytes(bytes: u64) -> (f64, &'static str) {
    match bytes {
        b if b < KILO_LIMIT => (bytes as f64, "B"),
        b if b < MEGA_LIMIT => (bytes as f64 / 1000.0, "KB"),
        b if b < GIGA_LIMIT => (bytes as f64 / 1_000_000.0, "MB"),
        b if b < TERA_LIMIT => (bytes as f64 / 1_000_000_000.0, "GB"),
        _ => (bytes as f64 / 1_000_000_000_000.0, "TB"),
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
pub fn truncate_to_text<'a, U: Into<usize>>(content: &str, width: U) -> Text<'a> {
    Text {
        lines: vec![Spans(vec![Span::raw(truncate_str(content, width))])],
    }
}

/// Returns the width of a str `s`. This takes into account some things like
/// joiners when calculating width.
pub fn str_width(s: &str) -> usize {
    UnicodeSegmentation::graphemes(s, true)
        .map(|g| {
            if g.contains('\u{200d}') {
                2
            } else {
                UnicodeWidthStr::width(g)
            }
        })
        .sum()
}

/// Returns the "width" of grapheme `g`. This takes into account some things like
/// joiners when calculating width.
///
/// Note that while you *can* pass in an entire string, the point is to check
/// individual graphemes (e.g. `"a"`, `"💎"`, `"大"`, `"🇨🇦"`).
#[inline]
fn grapheme_width(g: &str) -> usize {
    if g.contains('\u{200d}') {
        2
    } else {
        UnicodeWidthStr::width(g)
    }
}

/// Truncates a string with an ellipsis character.
///
/// NB: This probably does not handle EVERY case, but I think it handles most cases
/// we will use this function for fine... hopefully.
#[inline]
fn truncate_str<U: Into<usize>>(content: &str, width: U) -> String {
    let width = width.into();
    let mut text = String::with_capacity(width);

    if width > 0 {
        let mut curr_width = 0;
        let mut early_break = false;

        // This tracks the length of the last added string - note this does NOT match the grapheme *width*.
        let mut last_added_str_len = 0;

        // Cases to handle:
        // - Completes adding the entire string.
        // - Adds a character up to the boundary, then fails.
        // - Adds a character not up to the boundary, then fails.
        // Inspired by https://tomdebruijn.com/posts/rust-string-length-width-calculations/
        for g in UnicodeSegmentation::graphemes(content, true) {
            let g_width = grapheme_width(g);

            if curr_width + g_width <= width {
                curr_width += g_width;
                last_added_str_len = g.len();
                text.push_str(g);
            } else {
                early_break = true;
                break;
            }
        }

        if early_break {
            if curr_width == width {
                // Remove the last grapheme cluster added.
                text.truncate(text.len() - last_added_str_len);
            }
            text.push('…');
        }
    }

    text
}

#[inline]
pub const fn sort_partial_fn<T: std::cmp::PartialOrd>(is_descending: bool) -> fn(T, T) -> Ordering {
    if is_descending {
        partial_ordering_desc
    } else {
        partial_ordering
    }
}

/// Returns an [`Ordering`] between two [`PartialOrd`]s.
#[inline]
pub fn partial_ordering<T: std::cmp::PartialOrd>(a: T, b: T) -> Ordering {
    a.partial_cmp(&b).unwrap_or(Ordering::Equal)
}

/// Returns a reversed [`Ordering`] between two [`PartialOrd`]s.
///
/// This is simply a wrapper function around [`partial_ordering`] that reverses
/// the result.
#[inline]
pub fn partial_ordering_desc<T: std::cmp::PartialOrd>(a: T, b: T) -> Ordering {
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

    #[test]
    fn test_truncate() {
        let cpu_header = "CPU(c)▲";

        assert_eq!(
            truncate_str(cpu_header, 8_usize),
            cpu_header,
            "should match base string as there is enough room"
        );

        assert_eq!(
            truncate_str(cpu_header, 7_usize),
            cpu_header,
            "should match base string as there is enough room"
        );

        assert_eq!(truncate_str(cpu_header, 6_usize), "CPU(c…");
        assert_eq!(truncate_str(cpu_header, 5_usize), "CPU(…");
        assert_eq!(truncate_str(cpu_header, 4_usize), "CPU…");
        assert_eq!(truncate_str(cpu_header, 1_usize), "…");
        assert_eq!(truncate_str(cpu_header, 0_usize), "");
    }

    #[test]
    fn test_truncate_cjk() {
        let cjk = "施氏食獅史";

        assert_eq!(
            truncate_str(cjk, 11_usize),
            cjk,
            "should match base string as there is enough room"
        );

        assert_eq!(
            truncate_str(cjk, 10_usize),
            cjk,
            "should match base string as there is enough room"
        );

        assert_eq!(truncate_str(cjk, 9_usize), "施氏食獅…");
        assert_eq!(truncate_str(cjk, 8_usize), "施氏食…");
        assert_eq!(truncate_str(cjk, 2_usize), "…");
        assert_eq!(truncate_str(cjk, 1_usize), "…");
        assert_eq!(truncate_str(cjk, 0_usize), "");
    }

    #[test]
    fn test_truncate_mixed() {
        let test = "Test (施氏食獅史) Test";

        assert_eq!(
            truncate_str(test, 30_usize),
            test,
            "should match base string as there is enough room"
        );

        assert_eq!(
            truncate_str(test, 22_usize),
            test,
            "should match base string as there is just enough room"
        );

        assert_eq!(
            truncate_str(test, 21_usize),
            "Test (施氏食獅史) Te…",
            "should truncate the t and replace the s with ellipsis"
        );

        assert_eq!(truncate_str(test, 18_usize), "Test (施氏食獅史)…");
        assert_eq!(truncate_str(test, 17_usize), "Test (施氏食獅史…");
        assert_eq!(truncate_str(test, 16_usize), "Test (施氏食獅…");
        assert_eq!(truncate_str(test, 15_usize), "Test (施氏食獅…");
        assert_eq!(truncate_str(test, 14_usize), "Test (施氏食…");
        assert_eq!(truncate_str(test, 13_usize), "Test (施氏食…");
        assert_eq!(truncate_str(test, 8_usize), "Test (…");
        assert_eq!(truncate_str(test, 7_usize), "Test (…");
        assert_eq!(truncate_str(test, 6_usize), "Test …");
    }

    #[test]
    fn test_truncate_flags() {
        let flag = "🇨🇦";
        assert_eq!(truncate_str(flag, 3_usize), flag);
        assert_eq!(truncate_str(flag, 2_usize), flag);
        assert_eq!(truncate_str(flag, 1_usize), "…");
        assert_eq!(truncate_str(flag, 0_usize), "");

        let flag_text = "oh 🇨🇦";
        assert_eq!(truncate_str(flag_text, 6_usize), flag_text);
        assert_eq!(truncate_str(flag_text, 5_usize), flag_text);
        assert_eq!(truncate_str(flag_text, 4_usize), "oh …");

        let flag_text_wrap = "!🇨🇦!";
        assert_eq!(truncate_str(flag_text_wrap, 6_usize), flag_text_wrap);
        assert_eq!(truncate_str(flag_text_wrap, 4_usize), flag_text_wrap);
        assert_eq!(truncate_str(flag_text_wrap, 3_usize), "!…");
        assert_eq!(truncate_str(flag_text_wrap, 2_usize), "!…");
        assert_eq!(truncate_str(flag_text_wrap, 1_usize), "…");

        let flag_cjk = "加拿大🇨🇦";
        assert_eq!(truncate_str(flag_cjk, 9_usize), flag_cjk);
        assert_eq!(truncate_str(flag_cjk, 8_usize), flag_cjk);
        assert_eq!(truncate_str(flag_cjk, 7_usize), "加拿大…");
        assert_eq!(truncate_str(flag_cjk, 6_usize), "加拿…");
        assert_eq!(truncate_str(flag_cjk, 5_usize), "加拿…");
        assert_eq!(truncate_str(flag_cjk, 4_usize), "加…");

        let flag_mix = "🇨🇦加gaa拿naa大daai🇨🇦";
        assert_eq!(truncate_str(flag_mix, 20_usize), flag_mix);
        assert_eq!(truncate_str(flag_mix, 19_usize), "🇨🇦加gaa拿naa大daai…");
        assert_eq!(truncate_str(flag_mix, 18_usize), "🇨🇦加gaa拿naa大daa…");
        assert_eq!(truncate_str(flag_mix, 17_usize), "🇨🇦加gaa拿naa大da…");
        assert_eq!(truncate_str(flag_mix, 15_usize), "🇨🇦加gaa拿naa大…");
        assert_eq!(truncate_str(flag_mix, 14_usize), "🇨🇦加gaa拿naa…");
        assert_eq!(truncate_str(flag_mix, 13_usize), "🇨🇦加gaa拿naa…");
        assert_eq!(truncate_str(flag_mix, 3_usize), "🇨🇦…");
        assert_eq!(truncate_str(flag_mix, 2_usize), "…");
        assert_eq!(truncate_str(flag_mix, 1_usize), "…");
        assert_eq!(truncate_str(flag_mix, 0_usize), "");
    }

    /// This might not be the best way to handle it, but this at least tests that it doesn't crash...
    #[test]
    fn test_truncate_hindi() {
        // cSpell:disable
        let test = "हिन्दी";
        assert_eq!(truncate_str(test, 10_usize), test);
        assert_eq!(truncate_str(test, 6_usize), "हिन्दी");
        assert_eq!(truncate_str(test, 5_usize), "हिन्दी");
        assert_eq!(truncate_str(test, 4_usize), "हिन्…");
        assert_eq!(truncate_str(test, 3_usize), "हि…");
        assert_eq!(truncate_str(test, 2_usize), "…");
        assert_eq!(truncate_str(test, 1_usize), "…");
        assert_eq!(truncate_str(test, 0_usize), "");
        // cSpell:enable
    }

    #[test]
    fn test_truncate_emoji() {
        let heart = "❤️";
        assert_eq!(truncate_str(heart, 2_usize), heart);
        assert_eq!(truncate_str(heart, 1_usize), heart);
        assert_eq!(truncate_str(heart, 0_usize), "");

        let emote = "💎";
        assert_eq!(truncate_str(emote, 2_usize), emote);
        assert_eq!(truncate_str(emote, 1_usize), "…");
        assert_eq!(truncate_str(emote, 0_usize), "");

        let family = "👨‍👨‍👧‍👦";
        assert_eq!(truncate_str(family, 2_usize), family);
        assert_eq!(truncate_str(family, 1_usize), "…");
        assert_eq!(truncate_str(family, 0_usize), "");

        let scientist = "👩‍🔬";
        assert_eq!(truncate_str(scientist, 2_usize), scientist);
        assert_eq!(truncate_str(scientist, 1_usize), "…");
        assert_eq!(truncate_str(scientist, 0_usize), "");
    }
}
