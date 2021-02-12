// https://github.com/fdehau/tui-rs/blob/master/examples/popup.rs

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    style::{Style, Color},
    text::{Spans, Span},
    Frame,
};

pub enum UiModalType {
    Confirm
}

pub enum ModalOpt {
    Yes,
    No,
}

impl ModalOpt {
    pub fn toggle(&mut self) {
        match self {
            ModalOpt::Yes => *self = ModalOpt::No,
            ModalOpt::No => *self = ModalOpt::Yes,
        }
    }
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

pub fn draw_modal<B: Backend>(title: String, msg: String, sel_opt: &ModalOpt, f: &mut Frame<B>) {
    let block = Block::default().title(title).borders(Borders::ALL);
    let area = centered_rect(60, 20, f.size());



    f.render_widget(Clear, area); //this clears out the background
    f.render_widget(block, area);

    let mut yes_style = Style::default();
    let mut no_style = Style::default();

    match sel_opt {
        ModalOpt::Yes => yes_style.fg(Color::Green),
        ModalOpt::No => no_style.fg(Color::Green)
    };

    let span = Span::from(Span::styled("Yes", yes_style));
    let no_span = Span::from(Span::styled("No", no_style));

    let yes_btn = Paragraph::new(span).alignment(Alignment::Center);
    let no_btn = Paragraph::new(no_span).alignment(Alignment::Center);


    let modal_text = Paragraph::new(msg)
        .wrap(Wrap { trim: false });
    
    let btn_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(60),
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
        .split(btn_layout[1]);
    
    let text_layout = Layout::default()
        .margin(2)
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(10),
                Constraint::Percentage(80),
                Constraint::Percentage(10),
            ]
        )
        .split(btn_layout[0]);

    // let no_span = Paragraph::new("No");
    f.render_widget(modal_text, text_layout[1]);
    f.render_widget(yes_btn, btn_layout_x[1]);
    f.render_widget(no_btn, btn_layout_x[3]);
}
