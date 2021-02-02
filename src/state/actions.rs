use crate::network::host::HostMap;
use crate::network::ping_result::PingResult;

use std::net::IpAddr;

// TODO: create a Trait for AppActions

#[allow(dead_code)]
pub enum AppAction {
  BuildHosts(HostMap),
  UpdatePingResult(IpAddr, PingResult)
}
