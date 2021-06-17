use crossterm::{
    event,
    event::{read, poll, KeyCode, KeyEvent},
};

use std::sync::mpsc;
use std::thread;
use std::time::Duration;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Event {
    Key(Key),
    Tick,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Key {
    Char(char),
    Ctrl(char),
    Shift(char),
    Tab,
    BackTab,
    Up,
    Down,
    Left,
    Right,
    PageUp,
    PageDown,
    Backspace,
    Esc,
    Enter,
    Unknown
}

impl From<KeyEvent> for Key {
    fn from(ct_evt: KeyEvent) -> Self {
        match ct_evt {
            KeyEvent {
                code: KeyCode::Up,
                ..
            } => {
                Key::Up
            },

            KeyEvent {
                code: KeyCode::Down,
                ..
            } => {
                Key::Down
            },

            KeyEvent {
                code: KeyCode::Left,
                ..
            } => {
                Key::Left
            },

            KeyEvent {
                code: KeyCode::Right,
                ..
            } => {
                Key::Right
            },

            KeyEvent {
                code: KeyCode::PageUp,
                ..
            } => {
                Key::PageUp
            },

            KeyEvent {
                code: KeyCode::PageDown,
                ..
            } => {
                Key::PageDown
            },

            KeyEvent {
                code: KeyCode::Tab,
                ..
            } => {
                Key::Tab
            },

            KeyEvent {
                code: KeyCode::BackTab,
                ..
            } => {
                Key::BackTab
            },

            KeyEvent {
                code: KeyCode::Backspace,
                ..
            } => {
                Key::Backspace
            },

            KeyEvent {
                code: KeyCode::Esc,
                ..
            } => {
                Key::Esc
            },

            KeyEvent {
                code: KeyCode::Enter,
                ..
            } => {
                Key::Enter
            },

            KeyEvent {
                code: KeyCode::Char(c),
                modifiers: event::KeyModifiers::CONTROL
            } => {
                Key::Ctrl(c)
            },

            KeyEvent {
                code: KeyCode::Char(c),
                modifiers: event::KeyModifiers::SHIFT
            } => {
                Key::Shift(c)
            },

            KeyEvent {
                code: KeyCode::Char(c),
                ..
            } => {
                Key::Char(c)
            },

            _ => Key::Unknown
        }
    }
}

// TODO: this should use a findi event instead of key
pub struct EventReader {
    pub recv: mpsc::Receiver<Event>
}

pub fn async_event_reader(poll_interval: usize) -> EventReader {
    let (tx, rx) = mpsc::channel();

    let evt_tx = tx;
    thread::spawn(move || {
        loop {
            // Check for events at AppConfig.tick_len ms
            // if no events, send timer tick event
            // so the ui loop doesn't have to constantly spin
            if poll(Duration::from_millis(poll_interval as u64)).unwrap() {
                let evt = read().unwrap();
                if let event::Event::Key(kevt) = evt {
                    evt_tx.send(Event::Key(Key::from(kevt))).unwrap();
                }
            } else {
                evt_tx.send(Event::Tick).unwrap();
            }
        }
    });

    EventReader { recv: rx }
}
