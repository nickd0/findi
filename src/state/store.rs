use super::reducers::{AppReducer, Reducer};
use super::actions::AppAction;
use super::application_state::ApplicationState;

use std::sync::{Arc, Mutex};

pub type SharedAppStateStore = Arc<Mutex<AppStateStore>>;

pub struct AppStateStore {
  pub state: ApplicationState,
  dispatch_lock: Arc<Mutex<bool>>,
}

// TODO: need blocking read/write funcs for easier
// access to the store

// Add a AppStateStore Trait
impl AppStateStore {
  pub fn new() -> AppStateStore {
    AppStateStore {
      state: Default::default(),
      dispatch_lock: Default::default(),
    }
  }

  // Should this automatically spawn?
  // Should be non blocking, use async?
  // Could use async and then wait for the lock in the async fn

  // Need to potentially recover from a poisoned mutex here?
  // Use a mpsc queue here instead of blocking?
  pub fn dispatch(&mut self, action: AppAction) {
    let mut dplocked = self.dispatch_lock.lock().unwrap();
    *dplocked = true;
    self.state = AppReducer::reduce(action, self.state.clone());
    *dplocked = false;
  }

  pub fn dispatches(&mut self, actions: Vec<AppAction>) {
    let mut dplocked = self.dispatch_lock.lock().unwrap();
    *dplocked = true;
    for action in actions {
      self.state = AppReducer::reduce(action, self.state.clone());
    }
    *dplocked = false;
  }
}

// pub fn store_dispatch(store: SharedAppStateStore, action: AppAction) {
//   let mut lstore = store.lock().unwrap();
//   lstore.dispatch(action)
// }

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_dispatch_with_set_query_action() {
    let mut store = AppStateStore::new();
    let query = "Foo Query";
    store.dispatch(AppAction::SetQuery(query.to_string()));
    assert_eq!(store.state.query, query)
  }
}
