use tui::widgets::TableState;

use crate::network::host::{HostVec, Host};
use crate::ui::components::search_filter::SearchFilterOption;
use crate::ui::{
    pages::PageContent,
    notification::Notification,
    modal::Modal,
};
use crate::state::host_modal_state::HostModalState;

// use super::host_modal_state::HostModalState;

#[derive(Default, Clone)]
pub struct ApplicationState {
    pub hosts: HostVec,
    pub query: String,
    pub query_state: bool,
    pub input_err: bool,
    pub search_run: bool,
    pub curr_focus: PageContent,
    pub table_state: TableState,
    pub notification: Option<Notification>,
    pub modal: Option<Modal>,
    pub selected_host: Option<usize>,
    // TODO: make dynamic?
    pub modal_state: Option<HostModalState>,
    pub search_filter_opt: SearchFilterOption
    // TODO: should ui focus be part of application state?
    // pub focus: UiComponent
}

// State convenience methods
impl ApplicationState {
    pub fn get_selected_host(&self) -> Option<Host> {
        if let Some(idx) = self.selected_host {
            return Some(self.filtered_hosts().nth(idx).unwrap().clone())
        }

        None
    }

    pub fn filtered_hosts(&self) -> impl Iterator<Item = &Host> {
        self.hosts.iter().filter(move |&h| {
            if matches!(self.search_filter_opt, SearchFilterOption::ShowFound) {
                h.ping_res.is_some() || h.host_name.as_ref().unwrap_or(&Err(String::new())).is_ok()
            } else {
                true
            }
        })
    }
}
