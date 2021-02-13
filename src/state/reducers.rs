use super::actions::{Action, AppAction};
use super::application_state::ApplicationState;
use crate::network::host::{Host};
use crate::ui::pages::PageContent;

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

      AppAction::SetInputErr(err) => {
        state.input_err = err;
        state
      },

      AppAction::IterateFocus => {
        state
      },

      AppAction::SetHostSearchRun(run) => {
        state.search_run = run;
        state
      },

      AppAction::NewQuery(hosts) => {
        state.hosts = hosts.iter().map(|h| Host::new(*h) ).collect();
        state.search_run = true;
        state
      },

      AppAction::TableSelect(idx) => {
        state.table_state.select(Some(idx));
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

      _ => state
    }
  }
}
