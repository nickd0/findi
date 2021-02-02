use super::actions::AppAction;
use super::application_state::ApplicationState;
use crate::network::host::{Host};

pub enum AppReducer {
}

#[allow(dead_code)]
impl AppReducer {
  pub fn reduce(action: AppAction, mut state: ApplicationState) -> ApplicationState {
    match action {
      AppAction::BuildHosts(hosts) => {
        state.hosts = hosts.iter().map(|h| Host::new(*h) ).collect();
        return state
      },

      AppAction::UpdateHost(host) => {
        if let Some(idx) = state.hosts.iter().position(|&h| h.ip == host.ip) {
          state.hosts[idx] = host;
        }
        return state
      },

      _ => state
    }
  }
}
