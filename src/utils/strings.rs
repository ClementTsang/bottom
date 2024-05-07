use std::num::NonZeroUsize;

use tui::text::Text;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

/// Truncates text if it is too long, and adds an ellipsis at the end if needed.
///
/// TODO: Maybe cache results from this function for some cases? e.g. columns
#[inline]
pub fn truncate_to_text<'a, U: Into<usize>>(content: &str, width: U) -> Text<'a> {
    Text::raw(truncate_str(content, width))
}

/// Returns the width of a str `s`. This takes into account some things like
/// joiners when calculating width.
#[inline]
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

enum AsciiIterationResult {
    Complete,
    Remaining(usize),
}

/// Greedily add characters to the output until a non-ASCII grapheme is found, or
/// the output is `width` long.
#[inline]
fn greedy_ascii_add(content: &str, width: NonZeroUsize) -> (String, AsciiIterationResult) {
    let width: usize = width.into();

    const SIZE_OF_ELLIPSIS: usize = 3;
    let mut text = Vec::with_capacity(width - 1 + SIZE_OF_ELLIPSIS);

    let s = content.as_bytes();

    let mut current_index = 0;

    while current_index < width - 1 {
        let current_byte = s[current_index];
        if current_byte.is_ascii() {
            text.push(current_byte);
            current_index += 1;
        } else {
            debug_assert!(text.is_ascii());

            let current_index = AsciiIterationResult::Remaining(current_index);

            // SAFETY: This conversion is safe to do unchecked, we only push ASCII characters up to
            // this point.
            let current_text = unsafe { String::from_utf8_unchecked(text) };

            return (current_text, current_index);
        }
    }

    // If we made it all the way through, then we probably hit the width limit.
    debug_assert!(text.is_ascii());

    let current_index = if s[current_index].is_ascii() {
        let mut ellipsis = [0; SIZE_OF_ELLIPSIS];
        '…'.encode_utf8(&mut ellipsis);
        text.extend_from_slice(&ellipsis);
        AsciiIterationResult::Complete
    } else {
        AsciiIterationResult::Remaining(current_index)
    };

    // SAFETY: This conversion is safe to do unchecked, we only push ASCII characters up to
    // this point.
    let current_text = unsafe { String::from_utf8_unchecked(text) };

    (current_text, current_index)
}

/// Truncates a string to the specified width with an ellipsis character.
///
/// NB: This probably does not handle EVERY case, but I think it handles most cases
/// we will use this function for fine... hopefully.
///
/// TODO: Maybe fuzz this function?
/// TODO: Maybe release this as a lib? Testing against Fish's script [here](https://github.com/ridiculousfish/widecharwidth) might be useful.
#[inline]
fn truncate_str<U: Into<usize>>(content: &str, width: U) -> String {
    let width = width.into();

    if content.len() <= width {
        // If the entire string fits in the width, then we just
        // need to copy the entire string over.

        content.to_owned()
    } else if let Some(nz_width) = NonZeroUsize::new(width) {
        // What we are essentially doing is optimizing for the case that
        // most, if not all of the string is ASCII. As such:
        // - Step through each byte until (width - 1) is hit or we find a non-ascii
        //   byte.
        // - If the byte is ascii, then add it.
        //
        // If we didn't get a complete truncated string, then continue on treating the rest as graphemes.

        let (mut text, res) = greedy_ascii_add(content, nz_width);
        match res {
            AsciiIterationResult::Complete => text,
            AsciiIterationResult::Remaining(current_index) => {
                let mut curr_width = text.len();
                let mut early_break = false;

                // This tracks the length of the last added string - note this does NOT match the grapheme *width*.
                // Since the previous characters are always ASCII, this is always initialized as 1, unless the string
                // is empty.
                let mut last_added_str_len = if text.is_empty() { 0 } else { 1 };

                // Cases to handle:
                // - Completes adding the entire string.
                // - Adds a character up to the boundary, then fails.
                // - Adds a character not up to the boundary, then fails.
                // Inspired by https://tomdebruijn.com/posts/rust-string-length-width-calculations/
                for g in UnicodeSegmentation::graphemes(&content[current_index..], true) {
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
                text
            }
        }
    } else {
        String::default()
    }
}

/// Checks that the first string is equal to any of the other ones in a ASCII case-insensitive match.
///
/// The generated code is the same as writing:
/// `to_ascii_lowercase(a) == to_ascii_lowercase(b) || to_ascii_lowercase(a) == to_ascii_lowercase(c)`,
/// but without allocating and copying temporaries.
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
    use crate::utils::general::sort_partial_fn;

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
    fn test_truncate_str() {
        let cpu_header = "CPU(c)▲";

        assert_eq!(
            truncate_str(cpu_header, 8_usize),
            cpu_header,
            "should match base string as there is extra room"
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
    fn test_truncate_ascii() {
        let content = "0123456";

        assert_eq!(
            truncate_str(content, 8_usize),
            content,
            "should match base string as there is extra room"
        );

        assert_eq!(
            truncate_str(content, 7_usize),
            content,
            "should match base string as there is enough room"
        );

        assert_eq!(truncate_str(content, 6_usize), "01234…");
        assert_eq!(truncate_str(content, 5_usize), "0123…");
        assert_eq!(truncate_str(content, 4_usize), "012…");
        assert_eq!(truncate_str(content, 1_usize), "…");
        assert_eq!(truncate_str(content, 0_usize), "");
    }

    #[test]
    fn test_truncate_cjk() {
        let cjk = "施氏食獅史";

        assert_eq!(
            truncate_str(cjk, 11_usize),
            cjk,
            "should match base string as there is extra room"
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

        let cjk_2 = "你好嗎";
        assert_eq!(truncate_str(cjk_2, 5_usize), "你好…");
        assert_eq!(truncate_str(cjk_2, 4_usize), "你…");
        assert_eq!(truncate_str(cjk_2, 3_usize), "你…");
        assert_eq!(truncate_str(cjk_2, 2_usize), "…");
        assert_eq!(truncate_str(cjk_2, 1_usize), "…");
        assert_eq!(truncate_str(cjk_2, 0_usize), "");
    }

    #[test]
    fn test_truncate_mixed_one() {
        let test = "Test (施氏食獅史) Test";

        assert_eq!(
            truncate_str(test, 30_usize),
            test,
            "should match base string as there is extra room"
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

        assert_eq!(truncate_str(test, 20_usize), "Test (施氏食獅史) T…");
        assert_eq!(truncate_str(test, 19_usize), "Test (施氏食獅史) …");
        assert_eq!(truncate_str(test, 18_usize), "Test (施氏食獅史)…");
        assert_eq!(truncate_str(test, 17_usize), "Test (施氏食獅史…");
        assert_eq!(truncate_str(test, 16_usize), "Test (施氏食獅…");
        assert_eq!(truncate_str(test, 15_usize), "Test (施氏食獅…");
        assert_eq!(truncate_str(test, 14_usize), "Test (施氏食…");
        assert_eq!(truncate_str(test, 13_usize), "Test (施氏食…");
        assert_eq!(truncate_str(test, 8_usize), "Test (…");
        assert_eq!(truncate_str(test, 7_usize), "Test (…");
        assert_eq!(truncate_str(test, 6_usize), "Test …");
        assert_eq!(truncate_str(test, 5_usize), "Test…");
        assert_eq!(truncate_str(test, 4_usize), "Tes…");
    }

    #[test]
    fn test_truncate_mixed_two() {
        let test = "Test (施氏abc食abc獅史) Test";

        assert_eq!(
            truncate_str(test, 30_usize),
            test,
            "should match base string as there is extra room"
        );

        assert_eq!(
            truncate_str(test, 28_usize),
            test,
            "should match base string as there is just enough room"
        );

        assert_eq!(truncate_str(test, 26_usize), "Test (施氏abc食abc獅史) T…");
        assert_eq!(truncate_str(test, 21_usize), "Test (施氏abc食abc獅…");
        assert_eq!(truncate_str(test, 20_usize), "Test (施氏abc食abc…");
        assert_eq!(truncate_str(test, 16_usize), "Test (施氏abc食…");
        assert_eq!(truncate_str(test, 15_usize), "Test (施氏abc…");
        assert_eq!(truncate_str(test, 14_usize), "Test (施氏abc…");
        assert_eq!(truncate_str(test, 11_usize), "Test (施氏…");
        assert_eq!(truncate_str(test, 10_usize), "Test (施…");
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
    fn truncate_emoji() {
        let heart_1 = "♥";
        assert_eq!(truncate_str(heart_1, 2_usize), heart_1);
        assert_eq!(truncate_str(heart_1, 1_usize), heart_1);
        assert_eq!(truncate_str(heart_1, 0_usize), "");

        let heart_2 = "❤";
        assert_eq!(truncate_str(heart_2, 2_usize), heart_2);
        assert_eq!(truncate_str(heart_2, 1_usize), heart_2);
        assert_eq!(truncate_str(heart_2, 0_usize), "");

        // This one has a U+FE0F modifier at the end, and is thus considered "emoji-presentation",
        // see https://github.com/fish-shell/fish-shell/issues/10461#issuecomment-2079624670.
        // This shouldn't really be a common issue in a terminal but eh.
        let heart_emoji_pres = "❤️";
        assert_eq!(truncate_str(heart_emoji_pres, 2_usize), heart_emoji_pres);
        assert_eq!(truncate_str(heart_emoji_pres, 1_usize), "…");
        assert_eq!(truncate_str(heart_emoji_pres, 0_usize), "");

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
