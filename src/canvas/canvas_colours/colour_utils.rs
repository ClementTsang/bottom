use lazy_static::lazy_static;
use std::collections::HashMap;

use tui::style::{Color, Style};

use crate::utils::{error, gen_util::*};

const GOLDEN_RATIO: f32 = 0.618_034;
// Approx, good enough for use (also Clippy gets mad if it's too long)
pub const STANDARD_FIRST_COLOUR: Color = Color::LightMagenta;
pub const STANDARD_SECOND_COLOUR: Color = Color::LightYellow;
pub const STANDARD_THIRD_COLOUR: Color = Color::LightCyan;
pub const STANDARD_FOURTH_COLOUR: Color = Color::LightGreen;
pub const STANDARD_HIGHLIGHT_COLOUR: Color = Color::LightBlue;
pub const AVG_COLOUR: Color = Color::Red;

lazy_static! {
    static ref COLOR_NAME_LOOKUP_TABLE: HashMap<&'static str, Color> = [
        ("reset", Color::Reset),
        ("black", Color::Black),
        ("red", Color::Red),
        ("green", Color::Green),
        ("yellow", Color::Yellow),
        ("blue", Color::Blue),
        ("magenta", Color::Magenta),
        ("cyan", Color::Cyan),
        ("gray", Color::Gray),
        ("darkgray", Color::DarkGray),
        ("lightred", Color::LightRed),
        ("lightgreen", Color::LightGreen),
        ("lightyellow", Color::LightYellow),
        ("lightblue", Color::LightBlue),
        ("lightmagenta", Color::LightMagenta),
        ("lightcyan", Color::LightCyan),
        ("white", Color::White)
    ]
    .iter()
    .copied()
    .collect();
}

/// Generates random colours.  Strategy found from
/// https://martin.ankerl.com/2009/12/09/how-to-create-random-colors-programmatically/
pub fn gen_n_styles(num_to_gen: i32) -> Vec<Style> {
    fn gen_hsv(h: f32) -> f32 {
        let new_val = h + GOLDEN_RATIO;
        if new_val > 1.0 {
            new_val.fract()
        } else {
            new_val
        }
    }
    /// This takes in an h, s, and v value of range [0, 1]
    /// For explanation of what this does, see
    /// https://en.wikipedia.org/wiki/HSL_and_HSV#HSV_to_RGB_alternative
    fn hsv_to_rgb(hue: f32, saturation: f32, value: f32) -> (u8, u8, u8) {
        fn hsv_helper(num: u32, hu: f32, sat: f32, val: f32) -> f32 {
            let k = (num as f32 + hu * 6.0) % 6.0;
            val - val * sat * float_max(float_min(k, float_min(4.1 - k, 1.1)), 0.0)
        }

        (
            (hsv_helper(5, hue, saturation, value) * 255.0) as u8,
            (hsv_helper(3, hue, saturation, value) * 255.0) as u8,
            (hsv_helper(1, hue, saturation, value) * 255.0) as u8,
        )
    }

    // Generate colours
    // Why do we need so many colours?  Because macOS default terminal
    // throws a tantrum if you don't give it supported colours, but so
    // does PowerShell with some colours (Magenta and Yellow)!
    let mut colour_vec: Vec<Style> = vec![
        Style::default().fg(STANDARD_FIRST_COLOUR),
        Style::default().fg(STANDARD_SECOND_COLOUR),
        Style::default().fg(STANDARD_THIRD_COLOUR),
        Style::default().fg(STANDARD_FOURTH_COLOUR),
        Style::default().fg(Color::LightBlue),
        Style::default().fg(Color::LightRed),
        Style::default().fg(Color::Cyan),
        Style::default().fg(Color::Green),
        Style::default().fg(Color::Blue),
        Style::default().fg(Color::Red),
    ];

    let mut h: f32 = 0.4; // We don't need random colours... right?
    if num_to_gen - 10 > 0 {
        for _i in 0..(num_to_gen - 10) {
            h = gen_hsv(h);
            let result = hsv_to_rgb(h, 0.5, 0.95);
            colour_vec.push(Style::default().fg(Color::Rgb(result.0, result.1, result.2)));
        }
    }

    colour_vec
}

pub fn convert_hex_to_color(hex: &str) -> error::Result<Color> {
    fn convert_hex_to_rgb(hex: &str) -> error::Result<(u8, u8, u8)> {
        if hex.len() == 7 && &hex[0..1] == "#" {
            let r = u8::from_str_radix(&hex[1..3], 16)?;
            let g = u8::from_str_radix(&hex[3..5], 16)?;
            let b = u8::from_str_radix(&hex[5..7], 16)?;

            return Ok((r, g, b));
        }

        Err(error::BottomError::GenericError(format!(
            "Colour hex {} is not of valid length.  It must be a 7 character string of the form \"#112233\".",
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
        Err(error::BottomError::GenericError(format!(
            "Colour input {} is not valid.",
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
        Err(error::BottomError::GenericError(format!(
            "Colour input {} is not valid.",
            input_val
        )))
    }
}

pub fn get_style_from_hex(hex: &str) -> error::Result<Style> {
    Ok(Style::default().fg(convert_hex_to_color(hex)?))
}

fn convert_rgb_to_color(rgb_str: &str) -> error::Result<Color> {
    let rgb_list = rgb_str.split(',');
    let rgb = rgb_list
        .filter_map(|val| {
            if let Ok(res) = val.to_string().trim().parse::<u8>() {
                Some(res)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    if rgb.len() == 3 {
        Ok(Color::Rgb(rgb[0], rgb[1], rgb[2]))
    } else {
        Err(error::BottomError::GenericError(format!(
            "RGB colour {} is not of valid length.  It must be a comma separated value with 3 integers from 0 to 255, like \"255, 0, 155\".",
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

    Err(error::BottomError::GenericError(format!(
        "Color {} is not a supported config colour.  bottom supports the following named colours as strings: \
		Reset, Black, Red, Green, Yellow, Blue, Magenta, Cyan, Gray, DarkGray, LightRed, LightGreen, \
		LightYellow, LightBlue, LightMagenta, LightCyan, White",
        color_name
    )))
}

pub fn get_style_from_color_name(color_name: &str) -> error::Result<Style> {
    Ok(Style::default().fg(convert_name_to_color(color_name)?))
}
