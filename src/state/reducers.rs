use super::actions::{Action, AppAction};
use super::application_state::ApplicationState;
use crate::network::host::{Host};
use crate::ui::notification::Notification;

pub trait Reducer<T: Action> {
  fn reduce(action: T, state: ApplicationState) -> ApplicationState;
}

pub enum AppReducer {}

#[allow(dead_code)]
impl Reducer<AppAction> for AppReducer {
    fn reduce(action: AppAction, mut state: ApplicationState) -> ApplicationState {
        match action {
            AppAction::BuildHosts(hosts) => {
                state.hosts = hosts.iter().map(|h| Host::new(*h) ).collect();
                state
            },

            AppAction::UpdateHost(host) => {
                if let Some(idx) = state.hosts.iter().position(|h| h.ip == host.ip) {
                    state.hosts[idx] = host;
                }
                state
            },

            AppAction::SetQuery(query) => {
                state.query = query;
                state
            },

            AppAction::SetInputErr(err) => {
                state.input_err = err;
                state
            },

            AppAction::IterateFocus => {
                state
            },

            AppAction::SetHostSearchRun(run) => {
                state.search_run = run;
                if run {
                    let notif = Notification::info("Status", format!("Querying {} hosts...", state.hosts.len()).as_ref());
                    state.notification = Some(notif)
                }
                state
            },

            AppAction::NewQuery(hosts) => {
                state.hosts = hosts.iter().map(|h| Host::new(*h) ).collect();
                state.search_run = true;
                state
            },

            AppAction::TableSelect(idx) => {
                state.table_state.select(idx);
                state
            },

            AppAction::ShiftFocus(comp) => {
                state.curr_focus = comp;
                state
            },

            AppAction::SetNotification(notif) => {
                state.notification = notif;
                state
            },

            AppAction::SetModal(modal) => {
                state.modal = modal;
                state
            },

            AppAction::QueryComplete => {
                let notif = Notification::info("Status", "Host search complete");
                state.query_state = true;
                state.search_run = false;
                state.notification = Some(notif);
                state
            },

            AppAction::SetSearchFilter(opt) => {
                state.search_filter_opt = opt;
                state
            },

            _ => state
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::net::Ipv4Addr;

    const default_addr: Ipv4Addr = Ipv4Addr::new(10, 0, 0, 1);

    fn test_helper_reduce_state(action: AppAction, init_state: Option<ApplicationState>) -> ApplicationState {
        let state = match init_state {
            Some(state) => state,
            None => ApplicationState::default()
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
    fn test_action_update_host() {
        let host = Host::new(default_addr);
        let mut init_state = ApplicationState::default();
        init_state.hosts = vec![host];

        let mut updated_host = Host::new(default_addr);
        updated_host.ping_done = true;
        let action = AppAction::UpdateHost(updated_host);

        let new_state = test_helper_reduce_state(action, Some(init_state));

        assert_eq!(new_state.hosts[0].ip, default_addr);
        assert_eq!(new_state.hosts[0].ping_done, true)
    }

    #[test]
    fn test_action_set_search_run() {
        // Run ON
        let action = AppAction::SetHostSearchRun(true);
        let new_state = test_helper_reduce_state(action, None);
        assert_eq!(new_state.notification.unwrap().message, "Querying 0 hosts...".to_owned());

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
        assert_eq!(new_state.notification.unwrap().message, "Host search complete");
    }
}
