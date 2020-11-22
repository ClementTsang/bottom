use once_cell::sync::Lazy;
use std::collections::HashMap;

use tui::style::{Color, Style};

use crate::utils::error;

// Approx, good enough for use (also Clippy gets mad if it's too long)
pub const STANDARD_FIRST_COLOUR: Color = Color::LightMagenta;
pub const STANDARD_SECOND_COLOUR: Color = Color::LightYellow;
pub const STANDARD_THIRD_COLOUR: Color = Color::LightCyan;
pub const STANDARD_FOURTH_COLOUR: Color = Color::LightGreen;
pub const STANDARD_HIGHLIGHT_COLOUR: Color = Color::LightBlue;
pub const AVG_COLOUR: Color = Color::Red;
pub const ALL_COLOUR: Color = Color::Green;

static COLOR_NAME_LOOKUP_TABLE: Lazy<HashMap<&'static str, Color>> = Lazy::new(|| {
    [
        ("reset", Color::Reset),
        ("black", Color::Black),
        ("red", Color::Red),
        ("green", Color::Green),
        ("yellow", Color::Yellow),
        ("blue", Color::Blue),
        ("magenta", Color::Magenta),
        ("cyan", Color::Cyan),
        ("gray", Color::Gray),
        ("grey", Color::Gray),
        ("darkgray", Color::DarkGray),
        ("lightred", Color::LightRed),
        ("lightgreen", Color::LightGreen),
        ("lightyellow", Color::LightYellow),
        ("lightblue", Color::LightBlue),
        ("lightmagenta", Color::LightMagenta),
        ("lightcyan", Color::LightCyan),
        ("white", Color::White),
    ]
    .iter()
    .copied()
    .collect()
});

pub fn convert_hex_to_color(hex: &str) -> error::Result<Color> {
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

pub fn get_style_from_config(input_val: &str) -> error::Result<Style> {
    if input_val.len() > 1 {
        if &input_val[0..1] == "#" {
            get_style_from_hex(input_val)
        } else if input_val.contains(',') {
            get_style_from_rgb(input_val)
        } else {
            get_style_from_color_name(input_val)
        }
    } else {
        Err(error::BottomError::ConfigError(format!(
            "value \"{}\" is not valid.",
            input_val
        )))
    }
}

pub fn get_colour_from_config(input_val: &str) -> error::Result<Color> {
    if input_val.len() > 1 {
        if &input_val[0..1] == "#" {
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

pub fn get_style_from_hex(hex: &str) -> error::Result<Style> {
    Ok(Style::default().fg(convert_hex_to_color(hex)?))
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

pub fn get_style_from_rgb(rgb_str: &str) -> error::Result<Style> {
    Ok(Style::default().fg(convert_rgb_to_color(rgb_str)?))
}

fn convert_name_to_color(color_name: &str) -> error::Result<Color> {
    let color = COLOR_NAME_LOOKUP_TABLE.get(color_name.to_lowercase().as_str());
    if let Some(color) = color {
        return Ok(*color);
    }

    Err(error::BottomError::ConfigError(format!(
        "\"{}\" is an invalid named colour.
        
The following are supported strings: 
+--------+------------+--------------+
|  Reset | Magenta    | LightYellow  |
+--------+------------+--------------+
|  Black | Cyan       | LightBlue    |
+--------+------------+--------------+
|   Red  | Gray       | LightMagenta |
+--------+------------+--------------+
|  Green | DarkGray   | LightCyan    |
+--------+------------+--------------+
| Yellow | LightRed   | White        |
+--------+------------+--------------+
|  Blue  | LightGreen |              |
+--------+------------+--------------+
        ",
        color_name
    )))
}

pub fn get_style_from_color_name(color_name: &str) -> error::Result<Style> {
    Ok(Style::default().fg(convert_name_to_color(color_name)?))
}
