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
      }

      _ => state
    }
  }
}
