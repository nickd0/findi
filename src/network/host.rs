use super::tcp_ping::tcp_ping;
use super::udp_ping::udp_ping;
use super::ping_result::PingResult;

use std::net::{IpAddr};
use std::time::Duration;

pub enum PingType {
  UDP,
  TCP
};

pub struct Host {
  ip: IpAddr,
  tcp_ping: PingResult,
  udp_ping: PingResult,
  ping_time: Option<Duration>,
  ping_type: PingType
};

// TODO:
// a  user setting can indicate whether a tcp and/or a udp ping should be use
// also allow for ICMP echo
impl Host {
  pub fn ping() -> PingResult {
    match tcp_ping(self.ip) {
      Ok(t) =>
    }
  }
}
