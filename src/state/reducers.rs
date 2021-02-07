use super::actions::{Action, AppAction};
use super::application_state::ApplicationState;
use crate::network::host::{Host};

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
        return state
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

      AppAction::IterateFocus => {
        state
      }

      _ => state
    }
  }
}
