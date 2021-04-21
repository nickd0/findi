use crossterm::{
    event,
    event::{read, poll, KeyCode, KeyEvent},
};

use std::sync::mpsc;
use std::thread;
use std::time::Duration;

const POLL_INTERVAL: u64 = 100;

// enum Event {
//     ModalSelect(ModalOpt),
//     KeyboardEvent(Key)
// }


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
    pub recv: mpsc::Receiver<Key>
}

pub fn async_event_reader() -> EventReader {
    let (tx, rx) = mpsc::channel();

    let evt_tx = tx.clone();
    thread::spawn(move || {
        loop {
            if poll(Duration::from_millis(POLL_INTERVAL)).unwrap() {
                let evt = read().unwrap();
                match evt {
                    event::Event::Key(kevt) => {
                        evt_tx.send(Key::from(kevt)).unwrap();
                    },
                    _ => {}
                }
            }
        }
    });

    EventReader { recv: rx }
}
