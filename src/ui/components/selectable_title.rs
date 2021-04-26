use tui::{
    text::{Span, Spans},
    style::{Style, Modifier, Color}
};

pub fn selectable_title<'a>(title: &'a str, selected_style: Style) -> Spans<'a> {
    let (first, rest) = title.split_at(1);
    Spans::from(vec![
        Span::styled(first, Style::default().add_modifier(Modifier::UNDERLINED)),
        Span::styled(rest, selected_style)
    ])
}
