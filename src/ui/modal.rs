// https://github.com/fdehau/tui-rs/blob/master/examples/popup.rs

use crate::state::{
    actions::AppAction,
    store::{SharedAppStateStore, AppStateStore}
};
use termion::event::Key;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    style::{Style, Color},
    text::{Spans, Span},
    Frame,
};

#[derive(Clone)]
pub enum ModalType {
    YesNo,
    Info,
}

#[derive(Clone)]
pub struct Modal {
    pub modal_type: ModalType,
    pub title: String,
    pub message: String,
    pub selected: ModalOpt,
}

impl Modal {
    pub fn new(title: &str, message: &str, modal_type: ModalType) -> Self {
        Self {
            title: String::from(title),
            message: String::from(message),
            selected: ModalOpt::Yes,
            modal_type
        }
    }
}

pub enum UiModalType {
    Confirm
}

#[derive(Clone)]
pub enum ModalOpt {
    Yes,
    No,
}

// Make Modal Opt a trait for different options types
impl ModalOpt {
    pub fn toggle(&self) -> Self {
        match self {
            ModalOpt::Yes => ModalOpt::No,
            ModalOpt::No => ModalOpt::Yes,
        }
    }

    pub fn mut_toggle(&mut self) {
        *self = self.toggle()
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

pub fn draw_modal<B: Backend>(modal: Modal, f: &mut Frame<B>) {
    let block = Block::default().title(modal.title).borders(Borders::ALL);
    let area = centered_rect(40, 30, f.size());



    f.render_widget(Clear, area); //this clears out the background
    f.render_widget(block, area);

    let mut yes_style = Style::default();
    let mut no_style = Style::default();

    match modal.selected {
        ModalOpt::Yes => yes_style = yes_style.fg(Color::Green),
        ModalOpt::No => no_style = no_style.fg(Color::Green)
    };

    let span = Span::from(Span::styled("Yes", yes_style));
    let no_span = Span::from(Span::styled("No", no_style));

    let yes_btn = Paragraph::new(span).alignment(Alignment::Center);
    let no_btn = Paragraph::new(no_span).alignment(Alignment::Center);


    let msg_span = Spans::from(Span::from(modal.message));
    let modal_text = Paragraph::new(msg_span)
        .wrap(Wrap { trim: true });
    
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

pub fn handle_modal_event(key: Key, store: &mut AppStateStore) {
    match key {
        Key::BackTab | Key::Char('\t') => {
            let modal = store.state.modal.as_ref().unwrap();
            let mut mclone = modal.clone();
            mclone.selected.mut_toggle();
            store.dispatch(AppAction::SetModal(Some(mclone)))
        },

        Key::Char('\n') => {
            match store.state.modal.as_ref().unwrap().selected {
                ModalOpt::No => {
                    store.dispatch(AppAction::SetModal(None))
                },
                _ => {}
            }
        },

        Key::Esc => {
            store.dispatch(AppAction::SetModal(None))
        },

        _ => {}
    }
}
