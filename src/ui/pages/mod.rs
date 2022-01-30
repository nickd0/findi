use tui::{
    Frame,
    backend::Backend,
};

use crate::ui::event::Key;

use crate::state::store::{SharedAppStateStore, AppStateStore};

pub mod main_page;
pub mod service_page;

pub enum Page {
    MainPage,
    ServiceScanPage,
}

pub fn setup_page(curr_page: &Page, store: &mut AppStateStore) {
    #[allow(clippy::single_match)]  // Allow this while more pages are added
    match curr_page {
        Page::ServiceScanPage => service_page::setup_page(store),
        _ => {},
    }
}

pub fn draw_page<B: Backend>(curr_page: &Page, store: SharedAppStateStore, f: &mut Frame<B>) {
    match curr_page {
        Page::MainPage => main_page::draw_main_page(store, f),
        Page::ServiceScanPage => service_page::draw_page(store, f),
    }
}

pub fn handle_page_events(curr_page: &Page, key: Key, store: &mut AppStateStore, store_mtx: SharedAppStateStore) {
    match curr_page {
        Page::MainPage => main_page::handle_main_page_event(key, store, store_mtx),
        Page::ServiceScanPage => service_page::handle_page_event(key, store, store_mtx),
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PageContent {
    QueryInput,
    HostTable,
    SearchFilters,
    ServiceSelect,
}

impl Default for PageContent {
    fn default() -> PageContent {
        PageContent::HostTable
    }
}
