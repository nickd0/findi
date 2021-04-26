use tui::{
    widgets::{Paragraph, Block, Borders},
    style::{Style, Color},
    text::{Span, Spans}
};

pub enum InputStyleState {
    Focused,
    Error,
    Default
}

pub fn text_input<'a, T: Into<Spans<'a>>>(title: T, input: &'a str, style_state: InputStyleState) -> Paragraph<'a> {
    let selected_border_style = Style::default().fg(Color::Yellow);
    let default_border_style = Style::default().fg(Color::White);

    Paragraph::new(Span::from(input))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(
                    match style_state {
                        InputStyleState::Focused => selected_border_style,
                        InputStyleState::Error => selected_border_style.fg(Color::Red),
                        _ => default_border_style
                    }
                )
                .title(title)
        )
}
