use super::reducers::AppReducer;
use super::actions::AppAction;
use super::application_state::ApplicationState;

use std::sync::{Arc, Mutex};

pub type SharedAppStateStore = Arc<Mutex<AppStateStore>>;

pub struct AppStateStore {
  pub state: ApplicationState
}

// TODO: need blocking read/write funcs for easier
// access to the store
impl AppStateStore {
  pub fn new() -> AppStateStore {
    AppStateStore {
      state: Default::default()
    }
  }

  // Should this automatically spawn?
  // Should be non blocking, use async?
  // Could use async and then wait for the lock in the async fn
  pub fn dispatch(&mut self, action: AppAction) {
    self.state = AppReducer::reduce(action, self.state.clone());
  }
}
