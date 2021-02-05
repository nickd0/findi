use crate::network::host::HostVec;

#[derive(Default, Clone)]
pub struct ApplicationState {
  pub hosts: HostVec,
  pub query: String
}
