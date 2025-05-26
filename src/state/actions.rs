use crate::config::AppConfig;
use crate::network::host::Host;
use crate::network::ping_result::PingResult;
use crate::state::host_modal_state::HostModalAction;
use crate::ui::modal::Modal;
use crate::ui::notification::Notification;
use crate::ui::{components::search_filter::SearchFilterOption, pages::PageContent};

use std::net::Ipv4Addr;

pub trait Action {}

#[allow(dead_code)]
pub enum AppAction {
    BuildHosts(Vec<Ipv4Addr>),
    UpdatePingResult(Ipv4Addr, PingResult),
    UpdateHost(Host),
    SetQuery(String),
    SetPortQuery(Option<String>),
    SetInputErr(bool),
    SetHostSearchRun(bool),
    NewQuery(Vec<Ipv4Addr>),
    TableSelect(Option<usize>),
    ShiftFocus(PageContent),
    SetNotification(Option<Notification>),
    SetModal(Option<Modal>),
    SetSearchFilter(SearchFilterOption),
    SetSelectedHost(Option<usize>),
    SetModalAction(HostModalAction),
    QueryComplete,
    RestartQuery,
    IterateFocus,
    SetConfig(AppConfig),
    SetConfigNWorkers(usize),
    SetConfigTick(usize),
    SetNetworkNameServer(Ipv4Addr),
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
