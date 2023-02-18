use tui::style::{Color, Style};

use crate::utils::error;

pub const FIRST_COLOUR: Color = Color::LightMagenta;
pub const SECOND_COLOUR: Color = Color::LightYellow;
pub const THIRD_COLOUR: Color = Color::LightCyan;
pub const FOURTH_COLOUR: Color = Color::LightGreen;
pub const HIGHLIGHT_COLOUR: Color = Color::LightBlue;
pub const AVG_COLOUR: Color = Color::Red;
pub const ALL_COLOUR: Color = Color::Green;

fn convert_hex_to_color(hex: &str) -> error::Result<Color> {
    fn hex_err(hex: &str) -> error::Result<u8> {
        Err(
            error::BottomError::ConfigError(format!(
                "\"{}\" is an invalid hex colour.  It must be a valid 7 character hex string of the (ie: \"#112233\")."
            , hex))
        )
    }

    fn convert_hex_to_rgb(hex: &str) -> error::Result<(u8, u8, u8)> {
        let hex_components: Vec<char> = hex.chars().collect();

        if hex_components.len() == 7 {
            let mut r_string = hex_components[1].to_string();
            r_string.push(hex_components[2]);
            let mut g_string = hex_components[3].to_string();
            g_string.push(hex_components[4]);
            let mut b_string = hex_components[5].to_string();
            b_string.push(hex_components[6]);

            let r = u8::from_str_radix(&r_string, 16).or_else(|_err| hex_err(hex))?;
            let g = u8::from_str_radix(&g_string, 16).or_else(|_err| hex_err(hex))?;
            let b = u8::from_str_radix(&b_string, 16).or_else(|_err| hex_err(hex))?;

            return Ok((r, g, b));
        }

        Err(error::BottomError::ConfigError(format!(
            "\"{}\" is an invalid hex colour.  It must be a 7 character string of the form \"#112233\".",
            hex
        )))
    }

    let rgb = convert_hex_to_rgb(hex)?;
    Ok(Color::Rgb(rgb.0, rgb.1, rgb.2))
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
            convert_name_to_color(input_val)
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
            "value \"{}\" is an invalid RGB colour.  It must be a comma separated value with 3 integers from 0 to 255 (ie: \"255, 0, 155\").",
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
            "value \"{}\" contained invalid RGB values.  It must be a comma separated value with 3 integers from 0 to 255 (ie: \"255, 0, 155\").",
            rgb_str
        )))
    }
}

fn convert_name_to_color(color_name: &str) -> error::Result<Color> {
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
            "\"{}\" is an invalid named colour.
            
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
    fn test_invalid_colours() {
        // Test invalid spacing in single word.
        assert!(convert_name_to_color("bl ack").is_err());

        // Test invalid spacing in dual word.
        assert!(convert_name_to_color("darkg ray").is_err());

        // Test completely invalid colour.
        assert!(convert_name_to_color("darkreset").is_err());
    }

    #[test]
    fn test_valid_colours() {
        // Standard color should work
        assert_eq!(convert_name_to_color("red"), Ok(Color::Red));

        // Capitalizing should be fine.
        assert_eq!(convert_name_to_color("RED"), Ok(Color::Red));

        // Spacing shouldn't be an issue now.
        assert_eq!(convert_name_to_color(" red "), Ok(Color::Red));

        // The following are all equivalent.
        assert_eq!(convert_name_to_color("darkgray"), Ok(Color::DarkGray));
        assert_eq!(convert_name_to_color("darkgrey"), Ok(Color::DarkGray));
        assert_eq!(convert_name_to_color("dark grey"), Ok(Color::DarkGray));
        assert_eq!(convert_name_to_color("dark gray"), Ok(Color::DarkGray));

        assert_eq!(convert_name_to_color("grey"), Ok(Color::Gray));
        assert_eq!(convert_name_to_color("gray"), Ok(Color::Gray));

        // One more test with spacing.
        assert_eq!(
            convert_name_to_color(" lightmagenta "),
            Ok(Color::LightMagenta)
        );
        assert_eq!(
            convert_name_to_color("light magenta"),
            Ok(Color::LightMagenta)
        );
        assert_eq!(
            convert_name_to_color(" light magenta "),
            Ok(Color::LightMagenta)
        );
    }
}
