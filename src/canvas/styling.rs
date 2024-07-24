// #[cfg(test)]
// mod test {
//     use tui::style::{Color, Style};

//     use super::{CanvasStyles, ColourScheme};
//     use crate::options::Config;

//     #[test]
//     fn default_selected_colour_works() {
//         let mut colours = CanvasStyles::default();
//         let original_selected_text_colour = CanvasStyles::DEFAULT_SELECTED_TEXT_STYLE.fg.unwrap();
//         let original_selected_bg_colour = CanvasStyles::DEFAULT_SELECTED_TEXT_STYLE.bg.unwrap();

//         assert_eq!(
//             colours.selected_text_style,
//             Style::default()
//                 .fg(original_selected_text_colour)
//                 .bg(original_selected_bg_colour),
//         );

//         colours.set_selected_text_fg("red").unwrap();
//         assert_eq!(
//             colours.selected_text_style,
//             Style::default()
//                 .fg(Color::Red)
//                 .bg(original_selected_bg_colour),
//         );

//         colours.set_selected_text_bg("magenta").unwrap();
//         assert_eq!(
//             colours.selected_text_style,
//             Style::default().fg(Color::Red).bg(Color::Magenta),
//         );

//         colours.set_selected_text_fg("fake blue").unwrap_err();
//         assert_eq!(
//             colours.selected_text_style,
//             Style::default().fg(Color::Red).bg(Color::Magenta),
//         );

//         colours.set_selected_text_bg("fake blue").unwrap_err();
//         assert_eq!(
//             colours.selected_text_style,
//             Style::default().fg(Color::Red).bg(Color::Magenta),
//         );
//     }

//     #[test]
//     fn built_in_colour_schemes_work() {
//         let config = Config::default();
//         CanvasStyles::new(ColourScheme::Default, &config).unwrap();
//         CanvasStyles::new(ColourScheme::DefaultLight, &config).unwrap();
//         CanvasStyles::new(ColourScheme::Gruvbox, &config).unwrap();
//         CanvasStyles::new(ColourScheme::GruvboxLight, &config).unwrap();
//         CanvasStyles::new(ColourScheme::Nord, &config).unwrap();
//         CanvasStyles::new(ColourScheme::NordLight, &config).unwrap();
//     }
// }
