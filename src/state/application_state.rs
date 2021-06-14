use tui::widgets::TableState;

use crate::network::host::HostVec;
use crate::ui::components::search_filter::SearchFilterOption;
use crate::ui::{
    pages::PageContent,
    notification::Notification,
    modal::Modal,
};

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
    pub search_filter_opt: SearchFilterOption
    // TODO: should ui focus be part of application state?
    // pub focus: UiComponent
}
