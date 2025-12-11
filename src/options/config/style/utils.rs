use concat_string::concat_string;
use tui::style::Color;
use unicode_segmentation::UnicodeSegmentation;

/// Convert a hex string to a colour.
pub(super) fn try_hex_to_colour(hex: &str) -> Result<Color, String> {
    fn hex_component_to_int(hex: &str, first: &str, second: &str) -> Result<u8, String> {
        u8::from_str_radix(&concat_string!(first, second), 16)
            .map_err(|_| format!("'{hex}' is an invalid hex color, could not decode."))
    }

    fn invalid_hex_format(hex: &str) -> String {
        format!(
            "'{hex}' is an invalid hex color. It must be either a 7 character hex string of the form '#12ab3c' or a 3 character hex string of the form '#1a2'.",
        )
    }

    if !hex.starts_with('#') {
        return Err(invalid_hex_format(hex));
    }

    let components: Vec<&str> = hex.graphemes(true).collect();
    if components.len() == 7 {
        // A 6-long hex.
        let r = hex_component_to_int(hex, components[1], components[2])?;
        let g = hex_component_to_int(hex, components[3], components[4])?;
        let b = hex_component_to_int(hex, components[5], components[6])?;

        Ok(Color::Rgb(r, g, b))
    } else if components.len() == 4 {
        // A 3-long hex.
        let r = hex_component_to_int(hex, components[1], components[1])?;
        let g = hex_component_to_int(hex, components[2], components[2])?;
        let b = hex_component_to_int(hex, components[3], components[3])?;

        Ok(Color::Rgb(r, g, b))
    } else {
        Err(invalid_hex_format(hex))
    }
}

pub fn str_to_colour(input_val: &str) -> Result<Color, String> {
    if input_val.len() > 1 {
        if input_val.starts_with('#') {
            try_hex_to_colour(input_val)
        } else if input_val.contains(',') {
            convert_rgb_to_color(input_val)
        } else {
            convert_name_to_colour(input_val)
        }
    } else {
        Err(format!("Value '{input_val}' is not valid.",))
    }
}

fn convert_rgb_to_color(rgb_str: &str) -> Result<Color, String> {
    let rgb_list = rgb_str.split(',').collect::<Vec<&str>>();
    if rgb_list.len() != 3 {
        return Err(format!(
            "Value '{rgb_str}' is an invalid RGB colour. It must be a comma separated value with 3 integers from 0 to 255 (ie: '255, 0, 155').",
        ));
    }

    let rgb = rgb_list
        .iter()
        .filter_map(|val| (*(*val)).to_string().trim().parse::<u8>().ok())
        .collect::<Vec<_>>();

    if rgb.len() == 3 {
        Ok(Color::Rgb(rgb[0], rgb[1], rgb[2]))
    } else {
        Err(format!(
            "Value '{rgb_str}' contained invalid RGB values. It must be a comma separated value with 3 integers from 0 to 255 (ie: '255, 0, 155').",
        ))
    }
}

fn convert_name_to_colour(color_name: &str) -> Result<Color, String> {
    match color_name.to_lowercase().trim() {
        "reset" => Ok(Color::Reset),
        "black" => Ok(Color::Black),
        "red" => Ok(Color::Red),
        "green" => Ok(Color::Green),
        "yellow" => Ok(Color::Yellow),
        "blue" => Ok(Color::Blue),
        "magenta" => Ok(Color::Magenta),
        "cyan" => Ok(Color::Cyan),
        "gray" | "grey" => Ok(Color::Gray),
        "darkgray" | "darkgrey" | "dark gray" | "dark grey" => Ok(Color::DarkGray),
        "lightred" | "light red" => Ok(Color::LightRed),
        "lightgreen" | "light green" => Ok(Color::LightGreen),
        "lightyellow" | "light yellow" => Ok(Color::LightYellow),
        "lightblue" | "light blue" => Ok(Color::LightBlue),
        "lightmagenta" | "light magenta" => Ok(Color::LightMagenta),
        "lightcyan" | "light cyan" => Ok(Color::LightCyan),
        "white" => Ok(Color::White),
        _ => Err(format!(
            "'{color_name}' is an invalid named color.
            
The following are supported named colors: 
+--------+-------------+---------------------+
|  Reset | Magenta     | Light Yellow        |
+--------+-------------+---------------------+
|  Black | Cyan        | Light Blue          |
+--------+-------------+---------------------+
|   Red  | Gray/Grey   | Light Magenta       |
+--------+-------------+---------------------+
|  Green | Light Cyan  | Dark Gray/Dark Grey |
+--------+-------------+---------------------+
| Yellow | Light Red   | White               |
+--------+-------------+---------------------+
|  Blue  | Light Green |                     |
+--------+-------------+---------------------+

Alternatively, hex colors or RGB color codes are valid.\n"
        )),
    }
}

macro_rules! opt {
    ($($e: tt)+) => {
        (|| { $($e)+ })()
    }
}

macro_rules! set_style {
    ($palette_field:expr, $config_location:expr, $field:tt) => {
        if let Some(style) = &(opt!($config_location.as_ref()?.$field.as_ref())) {
            match &style {
                TextStyleConfig::Colour(colour) => {
                    $palette_field = $palette_field.fg(
                        crate::options::config::style::utils::str_to_colour(&colour.0).map_err(
                            |err| match stringify!($config_location).split_once(".") {
                                Some((_, loc)) => crate::options::OptionError::config(format!(
                                    "Please update 'styles.{loc}.{}' in your config file. {err}",
                                    stringify!($field)
                                )),
                                None => crate::options::OptionError::config(format!(
                                    "Please update 'styles.{}' in your config file. {err}",
                                    stringify!($field)
                                )),
                            },
                        )?,
                    );
                }
                TextStyleConfig::TextStyle {
                    color,
                    bg_color,
                    bold,
                    italics,
                } => {
                    if let Some(fg) = &color {
                        $palette_field = $palette_field
                            .fg(crate::options::config::style::utils::str_to_colour(
                            &fg.0,
                        )
                        .map_err(|err| {
                            match stringify!($config_location).split_once(".") {
                                Some((_, loc)) => crate::options::OptionError::config(format!(
                                    "Please update 'styles.{loc}.{}' in your config file. {err}",
                                    stringify!($field)
                                )),
                                None => crate::options::OptionError::config(format!(
                                    "Please update 'styles.{}' in your config file. {err}",
                                    stringify!($field)
                                )),
                            }
                        })?);
                    }

                    if let Some(bg) = &bg_color {
                        $palette_field = $palette_field
                            .bg(crate::options::config::style::utils::str_to_colour(
                            &bg.0,
                        )
                        .map_err(|err| {
                            match stringify!($config_location).split_once(".") {
                                Some((_, loc)) => crate::options::OptionError::config(format!(
                                    "Please update 'styles.{loc}.{}' in your config file. {err}",
                                    stringify!($field)
                                )),
                                None => crate::options::OptionError::config(format!(
                                    "Please update 'styles.{}' in your config file. {err}",
                                    stringify!($field)
                                )),
                            }
                        })?);
                    }

                    if let Some(bold) = &bold {
                        if *bold {
                            $palette_field =
                                $palette_field.add_modifier(tui::style::Modifier::BOLD);
                        } else {
                            $palette_field =
                                $palette_field.remove_modifier(tui::style::Modifier::BOLD);
                        }
                    }

                    if let Some(italics) = &italics {
                        if *italics {
                            $palette_field =
                                $palette_field.add_modifier(tui::style::Modifier::ITALIC);
                        } else {
                            $palette_field =
                                $palette_field.remove_modifier(tui::style::Modifier::ITALIC);
                        }
                    }
                }
            }
        }
    };
}

macro_rules! set_colour {
    ($palette_field:expr, $config_location:expr, $field:tt) => {
        if let Some(colour) = &(opt!($config_location.as_ref()?.$field.as_ref())) {
            $palette_field = $palette_field.fg(
                crate::options::config::style::utils::str_to_colour(&colour.0).map_err(|err| {
                    match stringify!($config_location).split_once(".") {
                        Some((_, loc)) => crate::options::OptionError::config(format!(
                            "Please update 'styles.{loc}.{}' in your config file. {err}",
                            stringify!($field)
                        )),
                        None => crate::options::OptionError::config(format!(
                            "Please update 'styles.{}' in your config file. {err}",
                            stringify!($field)
                        )),
                    }
                })?,
            );
        }
    };
}

macro_rules! set_colour_list {
    ($palette_field:expr, $config_location:expr, $field:tt) => {
        if let Some(colour_list) = &(opt!($config_location.as_ref()?.$field.as_ref())) {
            $palette_field = colour_list
                .iter()
                .map(|s| {
                    Ok(Style::default()
                        .fg(crate::options::config::style::utils::str_to_colour(&s.0)?))
                })
                .collect::<Result<Vec<Style>, String>>()
                .map_err(|err| match stringify!($config_location).split_once(".") {
                    Some((_, loc)) => crate::options::OptionError::config(format!(
                        "Please update 'styles.{loc}.{}' in your config file. {err}",
                        stringify!($field)
                    )),
                    None => crate::options::OptionError::config(format!(
                        "Please update 'styles.{}' in your config file. {err}",
                        stringify!($field)
                    )),
                })?;
        }
    };
}

pub(super) use opt;
pub(super) use set_colour;
pub(super) use set_colour_list;
pub(super) use set_style;

#[cfg(test)]
mod test {

    use tui::style::{Modifier, Style};

    use super::*;
    use crate::options::config::style::{ColorStr, TextStyleConfig};

    #[test]
    fn general_str_to_colour() {
        assert_eq!(str_to_colour("red").unwrap(), Color::Red);
        assert!(str_to_colour("r ed").is_err());

        assert_eq!(str_to_colour("#ffffff").unwrap(), Color::Rgb(255, 255, 255));
        assert!(str_to_colour("#fff fff").is_err());

        assert_eq!(
            str_to_colour("255, 255, 255").unwrap(),
            Color::Rgb(255, 255, 255)
        );
        assert!(str_to_colour("255, 256, 255").is_err());
    }

    #[test]
    fn invalid_colour_names() {
        // Test invalid spacing in single word.
        assert!(convert_name_to_colour("bl ack").is_err());

        // Test invalid spacing in dual word.
        assert!(convert_name_to_colour("darkg ray").is_err());

        // Test completely invalid colour.
        assert!(convert_name_to_colour("darkreset").is_err());
    }

    #[test]
    fn valid_colour_names() {
        // Standard color should work
        assert_eq!(convert_name_to_colour("red"), Ok(Color::Red));

        // Capitalizing should be fine.
        assert_eq!(convert_name_to_colour("RED"), Ok(Color::Red));

        // Spacing shouldn't be an issue now.
        assert_eq!(convert_name_to_colour(" red "), Ok(Color::Red));

        // The following are all equivalent.
        assert_eq!(convert_name_to_colour("darkgray"), Ok(Color::DarkGray));
        assert_eq!(convert_name_to_colour("darkgrey"), Ok(Color::DarkGray));
        assert_eq!(convert_name_to_colour("dark grey"), Ok(Color::DarkGray));
        assert_eq!(convert_name_to_colour("dark gray"), Ok(Color::DarkGray));

        assert_eq!(convert_name_to_colour("grey"), Ok(Color::Gray));
        assert_eq!(convert_name_to_colour("gray"), Ok(Color::Gray));

        // One more test with spacing.
        assert_eq!(
            convert_name_to_colour(" lightmagenta "),
            Ok(Color::LightMagenta)
        );
        assert_eq!(
            convert_name_to_colour("light magenta"),
            Ok(Color::LightMagenta)
        );
        assert_eq!(
            convert_name_to_colour(" light magenta "),
            Ok(Color::LightMagenta)
        );
    }

    #[test]
    fn valid_hex_colours() {
        // Check hex with 6 characters.
        assert_eq!(
            try_hex_to_colour("#ffffff").unwrap(),
            Color::Rgb(255, 255, 255)
        );
        assert_eq!(try_hex_to_colour("#000000").unwrap(), Color::Rgb(0, 0, 0));
        try_hex_to_colour("#111111").unwrap();
        try_hex_to_colour("#11ff11").unwrap();
        try_hex_to_colour("#1f1f1f").unwrap();
        assert_eq!(
            try_hex_to_colour("#123abc").unwrap(),
            Color::Rgb(18, 58, 188)
        );

        // Check hex with 3 characters.
        assert_eq!(
            try_hex_to_colour("#fff").unwrap(),
            Color::Rgb(255, 255, 255)
        );
        assert_eq!(try_hex_to_colour("#000").unwrap(), Color::Rgb(0, 0, 0));
        try_hex_to_colour("#111").unwrap();
        try_hex_to_colour("#1f1").unwrap();
        try_hex_to_colour("#f1f").unwrap();
        try_hex_to_colour("#ff1").unwrap();
        try_hex_to_colour("#1ab").unwrap();
        assert_eq!(try_hex_to_colour("#1ab").unwrap(), Color::Rgb(17, 170, 187));
    }

    #[test]
    fn invalid_hex_colours() {
        assert!(try_hex_to_colour("ffffff").is_err());
        assert!(try_hex_to_colour("111111").is_err());

        assert!(try_hex_to_colour("fff").is_err());
        assert!(try_hex_to_colour("111").is_err());
        assert!(try_hex_to_colour("fffffff").is_err());
        assert!(try_hex_to_colour("1234567").is_err());

        assert!(try_hex_to_colour("#fffffff").is_err());
        assert!(try_hex_to_colour("#1234567").is_err());
        assert!(try_hex_to_colour("#ff").is_err());
        assert!(try_hex_to_colour("#12").is_err());
        assert!(try_hex_to_colour("").is_err());

        assert!(try_hex_to_colour("#pppppp").is_err());
        assert!(try_hex_to_colour("#00000p").is_err());
        assert!(try_hex_to_colour("#ppp").is_err());

        assert!(try_hex_to_colour("#一").is_err());
        assert!(try_hex_to_colour("#一二").is_err());
        assert!(try_hex_to_colour("#一二三").is_err());
        assert!(try_hex_to_colour("#一二三四").is_err());

        assert!(try_hex_to_colour("#f一f").is_err());
        assert!(try_hex_to_colour("#ff一11").is_err());

        assert!(try_hex_to_colour("#🇨🇦").is_err());
        assert!(try_hex_to_colour("#🇨🇦🇨🇦").is_err());
        assert!(try_hex_to_colour("#🇨🇦🇨🇦🇨🇦").is_err());
        assert!(try_hex_to_colour("#🇨🇦🇨🇦🇨🇦🇨🇦").is_err());

        assert!(try_hex_to_colour("#हिन्दी").is_err());
    }

    #[test]
    fn test_rgb_colours() {
        assert_eq!(
            convert_rgb_to_color("0, 0, 0").unwrap(),
            Color::Rgb(0, 0, 0)
        );
        assert_eq!(
            convert_rgb_to_color("255, 255, 255").unwrap(),
            Color::Rgb(255, 255, 255)
        );
        assert!(convert_rgb_to_color("255, 256, 255").is_err());
        assert!(convert_rgb_to_color("256, 0, 256").is_err());
        assert!(convert_rgb_to_color("1, -1, 1").is_err());
        assert!(convert_rgb_to_color("1, -100000, 1").is_err());
        assert!(convert_rgb_to_color("1, -100000, 100000").is_err());
    }

    struct DummyConfig {
        inner: Option<InnerDummyConfig>,
    }

    struct InnerDummyConfig {
        color_a: Option<ColorStr>,
        color_b: Option<ColorStr>,
        color_c: Option<ColorStr>,
        color_d: Option<ColorStr>,
        many_colors: Option<Vec<ColorStr>>,
        text_a: Option<TextStyleConfig>,
        text_b: Option<TextStyleConfig>,
        text_c: Option<TextStyleConfig>,
        text_d: Option<TextStyleConfig>,
        text_e: Option<TextStyleConfig>,
        bad_color: Option<ColorStr>,
        bad_list: Option<Vec<ColorStr>>,
        bad_text_a: Option<TextStyleConfig>,
        bad_text_b: Option<TextStyleConfig>,
    }

    impl Default for InnerDummyConfig {
        fn default() -> Self {
            Self {
                color_a: None,
                color_b: Some(ColorStr("red".into())),
                color_c: Some(ColorStr("255, 255, 255".into())),
                color_d: Some(ColorStr("#000000".into())),
                many_colors: Some(vec![ColorStr("red".into()), ColorStr("blue".into())]),
                text_a: Some(TextStyleConfig::Colour(ColorStr("green".into()))),
                text_b: Some(TextStyleConfig::TextStyle {
                    color: None,
                    bg_color: None,
                    bold: None,
                    italics: None,
                }),
                text_c: Some(TextStyleConfig::TextStyle {
                    color: Some(ColorStr("magenta".into())),
                    bg_color: Some(ColorStr("255, 255, 255".into())),
                    bold: Some(true),
                    italics: Some(false),
                }),
                text_d: Some(TextStyleConfig::TextStyle {
                    color: Some(ColorStr("#fff".into())),
                    bg_color: Some(ColorStr("1, 1, 1".into())),
                    bold: Some(false),
                    italics: Some(true),
                }),
                text_e: None,
                bad_color: Some(ColorStr("asdf".into())),
                bad_list: Some(vec![
                    ColorStr("red".into()),
                    ColorStr("asdf".into()),
                    ColorStr("ghi".into()),
                ]),
                bad_text_a: Some(TextStyleConfig::TextStyle {
                    color: Some(ColorStr("asdf".into())),
                    bg_color: None,
                    bold: None,
                    italics: None,
                }),
                bad_text_b: Some(TextStyleConfig::TextStyle {
                    color: None,
                    bg_color: Some(ColorStr("asdf".into())),
                    bold: None,
                    italics: None,
                }),
            }
        }
    }

    #[test]
    fn test_set_colour() -> anyhow::Result<()> {
        let mut s = Style::default().fg(Color::Black);
        let dummy = DummyConfig {
            inner: Some(InnerDummyConfig::default()),
        };

        set_colour!(s, &dummy.inner, color_a);
        assert_eq!(s.fg.unwrap(), Color::Black);
        assert_eq!(s.bg, None);

        set_colour!(s, &dummy.inner, color_b);
        assert_eq!(s.fg.unwrap(), Color::Red);
        assert_eq!(s.bg, None);

        set_colour!(s, &dummy.inner, color_c);
        assert_eq!(s.fg.unwrap(), Color::Rgb(255, 255, 255));
        assert_eq!(s.bg, None);

        set_colour!(s, &dummy.inner, color_d);
        assert_eq!(s.fg.unwrap(), Color::Rgb(0, 0, 0));
        assert_eq!(s.bg, None);

        Ok(())
    }

    #[test]
    fn test_bad_set_colour() {
        let mut _s = Style::default().fg(Color::Black);
        let dummy = DummyConfig {
            inner: Some(InnerDummyConfig::default()),
        };

        (move || -> anyhow::Result<()> {
            set_colour!(_s, &dummy.inner, bad_color);

            Ok(())
        })()
        .unwrap_err();
    }

    #[test]
    fn test_set_multi_colours() -> anyhow::Result<()> {
        let mut s: Vec<Style> = vec![];
        let dummy = DummyConfig {
            inner: Some(InnerDummyConfig::default()),
        };

        set_colour_list!(s, &dummy.inner, many_colors);
        assert_eq!(s.len(), 2);
        assert_eq!(s[0].fg, Some(Color::Red));
        assert_eq!(s[1].fg, Some(Color::Blue));

        Ok(())
    }

    #[test]
    fn test_bad_set_list() {
        let mut _s: Vec<Style> = vec![];
        let dummy = DummyConfig {
            inner: Some(InnerDummyConfig::default()),
        };

        (|| -> anyhow::Result<()> {
            set_colour_list!(_s, &dummy.inner, bad_list);

            Ok(())
        })()
        .unwrap_err();
    }

    #[test]
    fn test_set_style() -> anyhow::Result<()> {
        let mut s = Style::default().fg(Color::Black);
        let dummy = DummyConfig {
            inner: Some(InnerDummyConfig::default()),
        };

        set_style!(s, &dummy.inner, text_e);
        assert_eq!(s.fg.unwrap(), Color::Black);
        assert_eq!(s.bg, None);
        assert!(s.add_modifier.is_empty());

        set_style!(s, &dummy.inner, text_a);
        assert_eq!(s.fg.unwrap(), Color::Green);
        assert_eq!(s.bg, None);

        set_style!(s, &dummy.inner, text_b);
        assert_eq!(s.fg.unwrap(), Color::Green);
        assert_eq!(s.bg, None);

        set_style!(s, &dummy.inner, text_c);
        assert_eq!(s.fg.unwrap(), Color::Magenta);
        assert_eq!(s.bg.unwrap(), Color::Rgb(255, 255, 255));
        assert!(s.add_modifier.contains(Modifier::BOLD));
        assert!(!s.add_modifier.contains(Modifier::ITALIC));

        set_style!(s, &dummy.inner, text_d);
        assert_eq!(s.fg.unwrap(), Color::Rgb(255, 255, 255));
        assert_eq!(s.bg.unwrap(), Color::Rgb(1, 1, 1));
        assert!(!s.add_modifier.contains(Modifier::BOLD));
        assert!(s.add_modifier.contains(Modifier::ITALIC));

        Ok(())
    }

    #[test]
    fn test_bad_text_1() {
        let mut _s = Style::default().fg(Color::Black);
        let dummy = DummyConfig {
            inner: Some(InnerDummyConfig::default()),
        };

        (move || -> anyhow::Result<()> {
            set_style!(_s, &dummy.inner, bad_text_a);

            Ok(())
        })()
        .unwrap_err();
    }

    #[test]
    fn test_bad_text_2() {
        let mut _s = Style::default().fg(Color::Black);
        let dummy = DummyConfig {
            inner: Some(InnerDummyConfig::default()),
        };
        (move || -> anyhow::Result<()> {
            set_style!(_s, &dummy.inner, bad_text_b);

            Ok(())
        })()
        .unwrap_err();
    }
}
