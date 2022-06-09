use tui::style::Style;

#[derive(Default)]
pub struct Styling {
    pub header_style: Style,
    pub border_style: Style,
    pub highlighted_border_style: Style,
    pub text_style: Style,
    pub highlighted_text_style: Style,
    pub title_style: Style,
}
