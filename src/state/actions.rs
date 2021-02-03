use crate::network::ping_result::PingResult;
use crate::network::host::Host;

use std::net::IpAddr;

pub trait Action {}

#[allow(dead_code)]
pub enum AppAction {
  BuildHosts(Vec<IpAddr>),
  UpdatePingResult(IpAddr, PingResult),
  UpdateHost(Host)
}

impl Action for AppAction {}
