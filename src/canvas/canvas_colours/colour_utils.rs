use crate::utils::{error, gen_util::*};
use tui::style::{Color, Style};

const GOLDEN_RATIO: f32 = 0.618_034; // Approx, good enough for use (also Clippy gets mad if it's too long)

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
	let mut colour_vec: Vec<Style> = vec![
		Style::default().fg(Color::LightMagenta),
		Style::default().fg(Color::LightYellow),
		Style::default().fg(Color::LightRed),
		Style::default().fg(Color::LightCyan),
		Style::default().fg(Color::LightGreen),
		Style::default().fg(Color::LightBlue),
	];

	let mut h: f32 = 0.4; // We don't need random colours... right?
	for _i in 0..(num_to_gen - 6) {
		h = gen_hsv(h);
		let result = hsv_to_rgb(h, 0.5, 0.95);
		colour_vec.push(Style::default().fg(Color::Rgb(result.0, result.1, result.2)));
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
