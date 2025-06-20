use super::actions::{Action, AppAction};
use super::application_state::ApplicationState;
use super::host_modal_state::{HostModalAction, HostModalState};
use crate::network::port_list::COMMON_PORTS;
use crate::network::{host::Host, tcp_ping::parse_portlist};

use crate::ui::{
    event::Key,
    modal::{Modal, ModalType},
    notification::Notification,
};

pub trait Reducer<T: Action> {
    fn reduce(action: T, state: ApplicationState) -> ApplicationState;
}

pub enum AppReducer {}

#[allow(dead_code)]
impl Reducer<AppAction> for AppReducer {
    fn reduce(action: AppAction, mut state: ApplicationState) -> ApplicationState {
        match action {
            AppAction::BuildHosts(hosts) => {
                state.hosts = hosts.iter().map(|h| Host::new(*h)).collect();
                state
            }

            // TODO: O(n), use a hashed data structure?
            AppAction::UpdateHost(host) => {
                if let Some(idx) = state.hosts.iter().position(|h| h.ip == host.ip) {
                    state.hosts[idx] = host;
                }
                state
            }

            AppAction::SetQuery(query) => {
                state.query = query;
                state
            }

            AppAction::SetPortQuery(pquery) => {
                match pquery {
                    Some(somepq) => match parse_portlist(&somepq) {
                        Ok(ports) => state.port_query = ports,
                        Err(_) => {}
                    },
                    None => state.port_query.clear(),
                }
                state
            }

            AppAction::SetInputErr(err) => {
                state.input_err = err;
                state
            }

            AppAction::IterateFocus => state,

            AppAction::SetHostSearchRun(run) => {
                state.search_run = run;
                if run {
                    let notif = Notification::info(
                        "Status",
                        format!("Querying {} hosts...", state.hosts.len()).as_ref(),
                    );
                    state.notification = Some(notif)
                }
                state
            }

            AppAction::NewQuery(hosts) => {
                state.hosts = hosts.iter().map(|h| Host::new(*h)).collect();
                state.search_run = true;
                state
            }

            AppAction::TableSelect(idx) => {
                state.table_state.select(idx);
                state
            }

            AppAction::ShiftFocus(comp) => {
                state.curr_focus = comp;
                state
            }

            AppAction::SetNotification(notif) => {
                state.notification = notif;
                state
            }

            AppAction::SetModal(modal) => {
                if modal.is_none() {
                    state.selected_host = None;
                    state.modal_state = None
                }
                state.modal = modal;
                state
            }

            AppAction::QueryComplete => {
                let notif = Notification::info("Status", "Host search complete");
                state.query_state = true;
                state.search_run = false;
                state.notification = Some(notif);
                state
            }

            AppAction::SetSearchFilter(opt) => {
                state.search_filter_opt = opt;
                state
            }

            AppAction::SetSelectedHost(host) => {
                state.selected_host = host;
                match state.selected_host {
                    Some(_) => {
                        state.modal = Some(Modal::new(
                            "Host info",
                            "Host information",
                            ModalType::Custom,
                        ));
                        state.modal_state =
                            Some(HostModalState::new(state.get_selected_host().unwrap()));
                    }
                    None => {
                        state.modal = None;
                        state.modal_state = None;
                    }
                }
                state
            }

            AppAction::SetModalAction(action) => {
                let mut modal_state = state.modal_state.clone().unwrap();
                match action {
                    HostModalAction::SetSelected(idx) => {
                        modal_state.tab_state.index = idx;
                        state.modal_state = Some(modal_state);
                    }

                    HostModalAction::SetPortQueryInput(key) => {
                        match key {
                            Key::Char(c) => modal_state.port_query.push(c),
                            Key::Backspace => {
                                let qlen = modal_state.port_query.len();
                                if qlen > 0 {
                                    modal_state.port_query =
                                        modal_state.port_query[..qlen - 1].to_string()
                                }
                            }
                            _ => {}
                        }

                        // TODO: don't need to do this every time an input event happens?
                        if let Ok(ports) = parse_portlist(&modal_state.port_query) {
                            // TODO: performance evaluation
                            modal_state.ports = ports.iter().map(|p| (*p, None)).collect();
                        }

                        state.modal_state = Some(modal_state);
                    }

                    HostModalAction::SetPortScanResult(res) => {
                        // TODO: Another O(n) operation
                        if let Some(idx) = modal_state.ports.iter().position(|p| p.0 == res.0) {
                            modal_state.ports[idx] = res;

                            // Add to active TCP ports
                            if let Some(Ok(_)) = res.1 {
                                if let Some(idx) = state
                                    .hosts
                                    .iter()
                                    .position(|h| h.ip == modal_state.selected_host.ip)
                                {
                                    state.hosts[idx].tcp_ports.insert(res.0);
                                }
                            }
                        } else {
                            modal_state.ports.push(res);
                        }

                        state.modal_state = Some(modal_state);
                    }

                    // TODO use this for custom port query
                    HostModalAction::SetCommonPortsForScanning => {
                        for port in COMMON_PORTS.iter() {
                            modal_state.ports.push((*port, None));
                        }
                        state.modal_state = Some(modal_state);
                    }
                }
                state
            }

            AppAction::SetConfig(config) => {
                state.app_config = config;
                state
            }

            AppAction::SetConfigNWorkers(nworkers) => {
                state.app_config.nworkers = nworkers;
                state
            }

            AppAction::SetConfigTick(tick_len) => {
                state.app_config.tick_len = tick_len;
                state
            }

            AppAction::SetNetworkNameServer(name_server) => {
                state.network_name_server = Some(name_server);
                state
            }
            _ => state,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::net::Ipv4Addr;

    const DEFAULT_ADDR: Ipv4Addr = Ipv4Addr::new(10, 0, 0, 1);

    fn test_helper_reduce_state(
        action: AppAction,
        init_state: Option<ApplicationState>,
    ) -> ApplicationState {
        let state = match init_state {
            Some(state) => state,
            None => ApplicationState::default(),
        };

        AppReducer::reduce(action, state.clone())
    }

    #[test]
    fn test_action_build_hosts() {
        let addr = Ipv4Addr::new(10, 0, 0, 1);
        let host = Host::new(addr);
        let action = AppAction::BuildHosts(vec![addr]);
        let new_state = test_helper_reduce_state(action, None);

        assert_eq!(new_state.hosts[0], host)
    }

    #[test]
    fn test_action_set_port_query() {
        let port_q = "10-12";
        let action = AppAction::SetPortQuery(Some(port_q.to_owned()));
        let new_state = test_helper_reduce_state(action, None);

        assert_eq!(new_state.port_query, vec![10, 11, 12]);
    }

    #[test]
    fn test_action_update_host() {
        let host = Host::new(DEFAULT_ADDR);
        let mut init_state = ApplicationState::default();
        init_state.hosts = vec![host];

        let mut updated_host = Host::new(DEFAULT_ADDR);
        updated_host.ping_done = true;
        let action = AppAction::UpdateHost(updated_host);

        let new_state = test_helper_reduce_state(action, Some(init_state));

        assert_eq!(new_state.hosts[0].ip, DEFAULT_ADDR);
        assert_eq!(new_state.hosts[0].ping_done, true)
    }

    #[test]
    fn test_action_set_search_run() {
        // Run ON
        let action = AppAction::SetHostSearchRun(true);
        let new_state = test_helper_reduce_state(action, None);
        assert_eq!(
            new_state.notification.unwrap().message,
            "Querying 0 hosts...".to_owned()
        );

        // Run OFF
        let action1 = AppAction::SetHostSearchRun(false);
        let new_state1 = test_helper_reduce_state(action1, None);
        assert_eq!(new_state1.notification.is_none(), true)
    }

    #[test]
    fn test_action_set_query_complete() {
        let action = AppAction::QueryComplete;
        let new_state = test_helper_reduce_state(action, None);

        assert_eq!(new_state.query_state, true);
        assert_eq!(new_state.search_run, false);
        assert_eq!(
            new_state.notification.unwrap().message,
            "Host search complete"
        );
    }

    #[test]
    fn test_action_set_selected_host() {
        let host_ip = Ipv4Addr::new(10, 0, 1, 1);
        let host = Host::new(host_ip);

        let mut init_state = ApplicationState::default();
        init_state.hosts = vec![host];

        let action = AppAction::SetSelectedHost(Some(0));
        let new_state = test_helper_reduce_state(action, Some(init_state));

        assert_eq!(new_state.get_selected_host().unwrap().ip, host_ip);

        assert_eq!(new_state.modal_state.is_some(), true);

        // Clear selected host
        let action = AppAction::SetSelectedHost(None);
        let new_state = test_helper_reduce_state(action, None);

        assert_eq!(new_state.get_selected_host(), None);
    }

    #[test]
    fn test_action_set_config_from_cli() {
        let nworkers = 50;
        let tick_len = 60;

        let action = AppAction::SetConfigNWorkers(nworkers);
        let new_state = test_helper_reduce_state(action, None);
        assert_eq!(new_state.app_config.nworkers, nworkers);

        let action = AppAction::SetConfigTick(tick_len);
        let new_state = test_helper_reduce_state(action, None);
        assert_eq!(new_state.app_config.tick_len, tick_len);
    }
}
