use tui::{
    Frame,
    backend::Backend,
};
use termion::event::{Key};

use crate::state::store::{SharedAppStateStore, AppStateStore};

pub mod main_page;

pub enum Page {
    MainPage,
}

pub fn draw_page<B: Backend>(curr_page: &Page, store: SharedAppStateStore, f: &mut Frame<B>) {
    match curr_page {
        Page::MainPage => main_page::draw_main_page(store, f)
    }
}

pub fn handle_page_events(curr_page: &Page, key: Key, store: &mut AppStateStore) {
    match curr_page {
        Page::MainPage => main_page::handle_main_page_event(key, store)
    }
}

#[derive(Copy, Clone)]
pub enum PageContent {
    QueryInput,
    HostTable,
    ConfirmModal
}

impl Default for PageContent {
    fn default() -> PageContent {
        PageContent::HostTable
    }
}

// pub fn handle_page_events(curr_page: &Page, key: Key) {
//     match curr_page {
//         Page::MainPage =>
//     }
// }
