use tui::widgets::TableState;

use crate::config::AppConfig;
use crate::network::host::{Host, HostVec};
use crate::state::host_modal_state::HostModalState;
use crate::ui::components::search_filter::SearchFilterOption;
use crate::ui::{modal::Modal, notification::Notification, pages::PageContent};

#[derive(Default, Clone)]
pub struct ApplicationState {
    pub hosts: HostVec,
    pub query: String,
    pub port_query: Vec<u16>,
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
    pub search_filter_opt: SearchFilterOption,
    pub app_config: AppConfig,
    // TODO: should ui focus be part of application state?
    // pub focus: UiComponent
}

// State convenience methods
impl ApplicationState {
    pub fn get_selected_host(&self) -> Option<Host> {
        if let Some(idx) = self.selected_host {
            return Some(self.filtered_hosts().nth(idx).unwrap().clone());
        }

        None
    }

    pub fn filtered_hosts(&self) -> impl Iterator<Item = &Host> {
        self.hosts
            .iter()
            .filter(move |&h| match self.search_filter_opt {
                SearchFilterOption::ShowFound => {
                    h.ping_res.is_some()
                        || h.host_name.as_ref().unwrap_or(&Err(String::new())).is_ok()
                }
                SearchFilterOption::ShowAll => true,
                SearchFilterOption::HasPort(idx) => h.tcp_ports.contains(&self.port_query[idx]),
            })
    }
}
