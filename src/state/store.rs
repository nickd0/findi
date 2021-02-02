use super::reducers::AppReducer;
use super::actions::AppAction;
use super::application_state::ApplicationState;

pub struct AppStateStore {
  state: ApplicationState
}

impl AppStateStore {
  pub fn new() -> AppStateStore {
    AppStateStore {
      state: Default::default()
    }
  }

  // Should this automatically spawn?
  pub fn dispatch(&mut self, action: AppAction) {
    self.state = AppReducer::reduce(action, self.state.clone());
  }
}
