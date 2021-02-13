// TODO: an unobtrusive notification in the upper right corner

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Clear, Paragraph},
    style::{Color, Modifier, Style},
    Frame,
};

#[derive(Copy, Clone)]
pub enum NotificationLevel {
    Info,
    Warn,
    Err
}

#[derive(Clone)]
pub struct Notification {
    pub title: String,
    pub message: String,
    pub level: NotificationLevel
}

impl Notification {
    pub fn new(title: &str, message: &str, level: NotificationLevel) -> Self {
        Self {
            title: title.to_owned(),
            message: message.to_owned(),
            level
        }
    }
}

fn cornered_rect(r: Rect) -> Rect {
    let percent_x = 20;
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(5),
                Constraint::Length(10),
                Constraint::Max(5)
            ]
            .as_ref(),
        )
        .split(r);
    
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(100 - percent_x),
                Constraint::Percentage(percent_x)
            ]
            .as_ref(),
        )
        .split(popup_layout[0])[1]
}

pub fn draw_notification<B: Backend>(notif: Notification, f: &mut Frame<B>) {
    let area = cornered_rect(f.size());
    let body = Paragraph::new(notif.message)
        .block(
            Block::default()
                .title(notif.title)
                .border_style(
                    Style::default()
                        .fg(
                            match notif.level {
                                NotificationLevel::Info => Color::LightBlue,
                                NotificationLevel::Warn => Color::LightRed,
                                NotificationLevel::Err => Color::Red
                            }
                        )
                )
                .borders(Borders::ALL)
        );
    f.render_widget(Clear, area); //this clears out the background
    f.render_widget(body, area);
}
