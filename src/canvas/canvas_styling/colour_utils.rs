use concat_string::concat_string;
use itertools::Itertools;
use tui::style::{Color, Style};
use unicode_segmentation::UnicodeSegmentation;

use crate::utils::error;

pub const FIRST_COLOUR: Color = Color::LightMagenta;
pub const SECOND_COLOUR: Color = Color::LightYellow;
pub const THIRD_COLOUR: Color = Color::LightCyan;
pub const FOURTH_COLOUR: Color = Color::LightGreen;
pub const HIGHLIGHT_COLOUR: Color = Color::LightBlue;
pub const AVG_COLOUR: Color = Color::Red;
pub const ALL_COLOUR: Color = Color::Green;

/// Convert a hex string to a colour.
fn convert_hex_to_color(hex: &str) -> error::Result<Color> {
    fn hex_component_to_int(hex: &str, first: &str, second: &str) -> error::Result<u8> {
        u8::from_str_radix(&concat_string!(first, second), 16).map_err(|_| {
            error::BottomError::ConfigError(format!(
                "\"{hex}\" is an invalid hex color, could not decode."
            ))
        })
    }

    fn invalid_hex_format(hex: &str) -> error::BottomError {
        error::BottomError::ConfigError(format!(
            "\"{hex}\" is an invalid hex color. It must be either a 7 character hex string of the form \"#12ab3c\" or a 3 character hex string of the form \"#1a2\".",
        ))
    }

    if !hex.starts_with('#') {
        return Err(invalid_hex_format(hex));
    }

    let components = hex.graphemes(true).collect_vec();
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

pub fn str_to_fg(input_val: &str) -> error::Result<Style> {
    Ok(Style::default().fg(str_to_colour(input_val)?))
}

pub fn str_to_colour(input_val: &str) -> error::Result<Color> {
    if input_val.len() > 1 {
        if input_val.starts_with('#') {
            convert_hex_to_color(input_val)
        } else if input_val.contains(',') {
            convert_rgb_to_color(input_val)
        } else {
            convert_name_to_colour(input_val)
        }
    } else {
        Err(error::BottomError::ConfigError(format!(
            "value \"{}\" is not valid.",
            input_val
        )))
    }
}

fn convert_rgb_to_color(rgb_str: &str) -> error::Result<Color> {
    let rgb_list = rgb_str.split(',').collect::<Vec<&str>>();
    if rgb_list.len() != 3 {
        return Err(error::BottomError::ConfigError(format!(
            "value \"{}\" is an invalid RGB colour. It must be a comma separated value with 3 integers from 0 to 255 (ie: \"255, 0, 155\").",
            rgb_str
        )));
    }

    let rgb = rgb_list
        .iter()
        .filter_map(|val| {
            if let Ok(res) = (*(*val)).to_string().trim().parse::<u8>() {
                Some(res)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    if rgb.len() == 3 {
        Ok(Color::Rgb(rgb[0], rgb[1], rgb[2]))
    } else {
        Err(error::BottomError::ConfigError(format!(
            "value \"{}\" contained invalid RGB values. It must be a comma separated value with 3 integers from 0 to 255 (ie: \"255, 0, 155\").",
            rgb_str
        )))
    }
}

fn convert_name_to_colour(color_name: &str) -> error::Result<Color> {
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
        _ => Err(error::BottomError::ConfigError(format!(
            "\"{}\" is an invalid named color.
            
The following are supported strings: 
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
        ",
            color_name
        ))),
    }
}

#[cfg(test)]
mod test {
    use super::*;

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
        assert_eq!(
            convert_hex_to_color("#ffffff").unwrap(),
            Color::Rgb(255, 255, 255)
        );
        assert_eq!(
            convert_hex_to_color("#000000").unwrap(),
            Color::Rgb(0, 0, 0)
        );
        convert_hex_to_color("#111111").unwrap();
        convert_hex_to_color("#11ff11").unwrap();
        convert_hex_to_color("#1f1f1f").unwrap();
        assert_eq!(
            convert_hex_to_color("#123abc").unwrap(),
            Color::Rgb(18, 58, 188)
        );

        assert_eq!(
            convert_hex_to_color("#fff").unwrap(),
            Color::Rgb(255, 255, 255)
        );
        assert_eq!(convert_hex_to_color("#000").unwrap(), Color::Rgb(0, 0, 0));
        convert_hex_to_color("#111").unwrap();
        convert_hex_to_color("#1f1").unwrap();
        convert_hex_to_color("#f1f").unwrap();
        convert_hex_to_color("#ff1").unwrap();
        convert_hex_to_color("#1ab").unwrap();
        assert_eq!(
            convert_hex_to_color("#1ab").unwrap(),
            Color::Rgb(17, 170, 187)
        );
    }

    #[test]
    fn invalid_hex_colours() {
        assert!(convert_hex_to_color("ffffff").is_err());
        assert!(convert_hex_to_color("111111").is_err());

        assert!(convert_hex_to_color("fff").is_err());
        assert!(convert_hex_to_color("111").is_err());
        assert!(convert_hex_to_color("fffffff").is_err());
        assert!(convert_hex_to_color("1234567").is_err());

        assert!(convert_hex_to_color("#fffffff").is_err());
        assert!(convert_hex_to_color("#1234567").is_err());
        assert!(convert_hex_to_color("#ff").is_err());
        assert!(convert_hex_to_color("#12").is_err());
        assert!(convert_hex_to_color("").is_err());

        assert!(convert_hex_to_color("#pppppp").is_err());
        assert!(convert_hex_to_color("#00000p").is_err());
        assert!(convert_hex_to_color("#ppp").is_err());

        assert!(convert_hex_to_color("#一").is_err());
        assert!(convert_hex_to_color("#一二").is_err());
        assert!(convert_hex_to_color("#一二三").is_err());
        assert!(convert_hex_to_color("#一二三四").is_err());

        assert!(convert_hex_to_color("#f一f").is_err());
        assert!(convert_hex_to_color("#ff一11").is_err());

        assert!(convert_hex_to_color("#🇨🇦").is_err());
        assert!(convert_hex_to_color("#🇨🇦🇨🇦").is_err());
        assert!(convert_hex_to_color("#🇨🇦🇨🇦🇨🇦").is_err());
        assert!(convert_hex_to_color("#🇨🇦🇨🇦🇨🇦🇨🇦").is_err());

        assert!(convert_hex_to_color("#हिन्दी").is_err());
    }
}
