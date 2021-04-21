use super::store::AppStateStore;

use crate::ui::event::Key;

pub trait ModalStateType: Clone {
    fn handle_event(&self, key: Key, store: &mut AppStateStore);
    // fn render
}

/// TODO move these elsewhere
/// The idea here is to use one struct type to hold
/// modal state for any type of modal. Then use a function that accepts
/// a struct of trait ModalStateType to handle the event and render the modal
/// Could expand this into a component system?
pub enum ModalRenderer {
    Confirm,
    Host
}

#[derive(Clone)]
pub struct ModalState {
    tabs: Vec<String>,
    buttons: Vec<String>,
    selected_tab: usize
}
