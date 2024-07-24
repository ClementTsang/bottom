use concat_string::concat_string;
use tui::style::{Color, Style};
use unicode_segmentation::UnicodeSegmentation;

/// Convert a hex string to a colour.
fn convert_hex_to_color(hex: &str) -> Result<Color, String> {
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

pub fn str_to_colour(input_val: &str) -> Result<Color, String> {
    if input_val.len() > 1 {
        if input_val.starts_with('#') {
            convert_hex_to_color(input_val)
        } else if input_val.contains(',') {
            convert_rgb_to_color(input_val)
        } else {
            convert_name_to_colour(input_val)
        }
    } else {
        Err(format!("Value '{input_val}' is not valid.",))
    }
}

pub(super) fn str_to_fg(input_val: &str) -> Result<Style, String> {
    Ok(Style::default().fg(str_to_colour(input_val)?))
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
+--------+-------------+---------------------+\n"
        )),
    }
}

macro_rules! opt {
    ($($e: tt)+) => {
        (|| { $($e)+ })()
    }
}

macro_rules! try_set_style {
    ($palette_field:expr, $option_field:expr) => {
        if let Some(style) = &$option_field {
            if let Some(colour) = style.color {
                $palette_field = crate::options::config::style::utils::str_to_fg(&colour.0)
                    .map_err(|err| {
                        OptionError::config(format!(
                            "Please update 'colors.{}' in your config file. {err}",
                            stringify!($colour_field)
                        ))
                    })?;
            }
        }
    };
}

macro_rules! try_set_colour {
    ($palette_field:expr, $option_colour:expr) => {
        if let Some(colour) = &$option_colour {
            $palette_field =
                crate::options::config::style::utils::str_to_fg(&colour.0).map_err(|err| {
                    OptionError::config(format!(
                        "Please update 'colors.{}' in your config file. {err}",
                        stringify!($colour_field)
                    ))
                })?;
        }
    };
}

macro_rules! try_set_colour_list {
    ($palette_field:expr, $option_field:expr) => {
        if let Some(colour_list) = &$option_field {
            $palette_field = colour_list
                .iter()
                .map(|s| crate::options::config::style::utils::str_to_fg(&s.0))
                .collect::<Result<Vec<Style>, String>>()
                .map_err(|err| {
                    OptionError::config(format!(
                        "Please update 'colors.{}' in your config file. {err}",
                        stringify!($colour_field)
                    ))
                })?;
        }
    };
}

pub(super) use {opt, try_set_colour, try_set_colour_list, try_set_style};

#[cfg(test)]
mod test {}
