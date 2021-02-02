use super::actions::AppAction;
use super::application_state::ApplicationState;

pub enum AppReducer {
}

#[allow(dead_code)]
impl AppReducer {
  pub fn reduce(action: AppAction, mut state: ApplicationState) -> ApplicationState {
    match action {
      AppAction::BuildHosts(hosts) => {
        state.hosts = hosts;
        return state
      },
      _ => state
    }
  }
}
