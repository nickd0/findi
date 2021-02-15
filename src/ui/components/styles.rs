use tui::style::{Style, Color};

// TODO: use config here for custom styles
pub fn border_style(selected: bool) -> Style {
    Style::default()
        .fg(
            if selected {
                Color::Yellow
            } else {
                Color::White
            }
        )
}
