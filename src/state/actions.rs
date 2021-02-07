use crate::network::ping_result::PingResult;
use crate::network::host::Host;

use std::net::Ipv4Addr;

pub trait Action {}

#[allow(dead_code)]
pub enum AppAction {
  BuildHosts(Vec<Ipv4Addr>),
  UpdatePingResult(Ipv4Addr, PingResult),
  UpdateHost(Host),
  SetQuery(String),
  IterateFocus
}

impl Action for AppAction {}
