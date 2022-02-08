// Stateful table impl.

use tui::widgets::TableState;
use crate::ui::event::Key;


const JUMP_LEN: usize = 20;

pub struct StatefulTable<'a, T> {
  pub state: &'a TableState,
  pub items: &'a Vec<T>
}

impl<'a, T> StatefulTable<'a, T> {

    #[allow(clippy::ptr_arg)]
    pub fn new(state: &'a TableState, items: &'a Vec<T>) -> StatefulTable<'a, T> {
        StatefulTable {
            state,
            items
        }
    }

    pub fn handle_key(&mut self, key: Key) -> Option<usize> {
        match key {
            Key::Down => self.next(),
            Key::Up => self.prev(),
            _ => None,
        }
    }

    pub fn next(&self) -> Option<usize> {
        match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    Some(0)
                } else {
                    Some(i + 1)
                }
            },
            None => Some(0)
        }
    }

    pub fn prev(&self) -> Option<usize> {
        match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    Some(self.items.len() - 1)
                } else {
                    Some(i - 1)
                }
            },
            None => Some(self.items.len() - 1)
        }
    }

    // TODO use table window size to paginate
    pub fn pgdn(&self) -> Option<usize> {
        let idx = match self.state.selected() {
            Some(i) => {
                if i + JUMP_LEN > self.items.len() - 1 {
                    0
                } else {
                    i + JUMP_LEN
                }
            },
            None => 0
        };
        Some(idx)
    }

    pub fn pgup(&self) -> Option<usize> {
        let idx = match self.state.selected() {
            Some(i) => {
                if i < JUMP_LEN {
                    self.items.len() -1
                } else {
                    i - JUMP_LEN
                }
            },
            None => 0
        };
        Some(idx)
    }
}
