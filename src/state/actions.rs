use crate::network::ping_result::PingResult;
use crate::network::host::Host;

use std::net::IpAddr;

// TODO: create a Trait for AppActions

#[allow(dead_code)]
pub enum AppAction {
  BuildHosts(Vec<IpAddr>),
  UpdatePingResult(IpAddr, PingResult),
  UpdateHost(Host)
}
