use super::tcp_ping::tcp_ping;
use super::udp_ping::udp_ping;
use super::ping_result::PingResult;

use std::net::{IpAddr};
use std::fmt;

pub enum PingType {
  UDP,
  TCP
}

impl fmt::Display for PingType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
    match *self {
      PingType::UDP => { write!(f, "UDP ping") },
      PingType::TCP => { write!(f, "TCP ping") }
    }
  }
}

pub struct Host {
  pub ip: IpAddr,
  pub ping_res: Option<PingResult>,
  pub ping_type: Option<PingType>
}

// TODO:
// a  user setting can indicate whether a tcp and/or a udp ping should be use
// also allow for ICMP echo
impl Host {
  pub fn new(ip: IpAddr) -> Host {
    Host {
      ip: ip,
      ping_res: None,
      ping_type: None
    }
  }

  pub fn ping(&mut self) {
    self.ping_res = match udp_ping(self.ip) {
      Ok(t) => {
        self.ping_type = Some(PingType::UDP);
        Some(Ok(t))
      },
      Err(_) => {
        match tcp_ping(self.ip) {
          Ok(t) => {
            self.ping_type = Some(PingType::TCP);
            Some(Ok(t))
          },
          Err(e) => { Some(Err(e)) }
        }
      }
    };
  }
}
