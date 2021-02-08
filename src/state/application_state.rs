use crate::network::host::HostVec;

#[derive(Default, Clone)]
pub struct ApplicationState {
  pub hosts: HostVec,
  pub query: String,
  pub input_err: bool,
  // TODO: should ui focus be part of application state?
  // pub focus: UiComponent
}
