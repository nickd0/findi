// TODO: an unobtrusive notification in the upper right corner

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    style::{Color, Style},
    text::{Spans, Span},
    Frame,
};

#[derive(Copy, Clone)]
pub enum NotificationLevel {
    Info,
    Warn,
}

#[derive(Clone)]
pub struct Notification {
    pub title: String,
    pub message: String,
    pub level: NotificationLevel
}

impl Default for Notification {
    fn default() -> Self {
        Notification {
            title: "Status".to_owned(),
            message: String::default(),
            level: NotificationLevel::Info
        }
    }
}

impl Notification {
    pub fn info(title: &str, message: &str) -> Self {
        Self {
            title: title.to_owned(),
            message: message.to_owned(),
            level: NotificationLevel::Info
        }
    }

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

    let msg_span = Spans::from(Span::from(notif.message));
    let body = Paragraph::new(msg_span)
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .title(notif.title)
                .border_style(
                    Style::default()
                        .fg(
                            match notif.level {
                                NotificationLevel::Info => Color::LightBlue,
                                NotificationLevel::Warn => Color::LightRed,
                            }
                        )
                )
                .borders(Borders::ALL)
        );
    f.render_widget(Clear, area); //this clears out the background
    f.render_widget(body, area);
}
