use super::store::SharedAppStateStore;

use crate::network::ping_result::PingResult;
use crate::network::{
    host::Host,
};
use crate::ui::{
    pages::PageContent,
    components::search_filter::SearchFilterOption
};
use crate::ui::notification::Notification;
use crate::ui::modal::Modal;

use std::net::Ipv4Addr;

pub trait Action {}

#[allow(dead_code)]
pub enum AppAction {
    BuildHosts(Vec<Ipv4Addr>),
    UpdatePingResult(Ipv4Addr, PingResult),
    UpdateHost(Host),
    SetQuery(String),
    SetInputErr(bool),
    SetHostSearchRun(bool),
    NewQuery(Vec<Ipv4Addr>),
    TableSelect(usize),
    ShiftFocus(PageContent),
    SetNotification(Option<Notification>),
    SetModal(Option<Modal>),
    SetSearchFilter(SearchFilterOption),
    QueryComplete,
    RestartQuery,
    IterateFocus
}

// impl AppAction {
//     // Return something callable, use middleware to pass a share store state
//     // handle to the closure that is returned
//     pub fn start_host_query() -> Box<dyn Fn(SharedAppStateStore)> {
//         Box::new(|lstore: SharedAppStateStore| {
//             init_host_search(lstore.clone());
//             lstore.lock().unwrap().dispatch(AppAction::SetHostSearchRun(true))
//         })
//     }
// }

impl Action for AppAction {}
