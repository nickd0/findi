// https://github.com/fdehau/tui-rs/blob/master/examples/popup.rs

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    widgets::{Block, Borders, Clear, Paragraph},
    style::{Style, Color, Modifier},
    text::{Spans, Span},
    Frame,
};

pub enum UiModalType {
    Confirm
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

pub fn draw_modal<B: Backend>(title: String, f: &mut Frame<B>) {
    let block = Block::default().title(title).borders(Borders::ALL);
    let area = centered_rect(60, 20, f.size());



    f.render_widget(Clear, area); //this clears out the background
    f.render_widget(block, area);

    let span = Span::from(Span::styled("Yes", Style::default().fg(Color::Green)));
    let no_span = Span::from(Span::styled("No", Style::default()));

    let yes_btn = Paragraph::new(span);
    let no_btn = Paragraph::new(no_span);
    
    let btn_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
            ]
            .as_ref(),
        )
        .split(area);

    let btn_layout_x = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
            ]
            .as_ref(),
        )
        .split(btn_layout[3]);

    // let no_span = Paragraph::new("No");
    f.render_widget(yes_btn, btn_layout_x[1]);
    f.render_widget(no_btn, btn_layout_x[3]);
}
