use crate::network::host::HostMap;

#[derive(Default, Clone)]
pub struct ApplicationState {
  pub hosts: HostMap
}
