use tui::{
    Frame,
    backend::{Backend, TermionBackend},
};

use super::page::UiPage;
use super::modal;

#[derive(Debug)]
pub enum MainPageContent {
    QueryInput,
    HostsTable,
    ConfirmModal
}

// pub struct MainPage {
// }

// impl MainPage {
//     pub fn new() -> Self {
//         Self {}
//     }
// }

// impl<B: Backend> UiPage<B> for MainPage {
//     fn draw(&self, f: &mut Frame<B>) {
//         let mut opt = modal::ModalOpt::Yes;
//         modal::draw_modal("Modal".to_owned(), "Modal text?".to_owned(), &opt, f);
//     }
// }

// Make a macro
// impl UiComponent {
//     pub fn iterator() -> Iter<'static, UiComponent> {
//         static DIRECTIONS: [UiComponent; 4] = [QueryInput, HostsTable, ProgresGauge]
//         DIRECTIONS.iter()
//     }
// }
