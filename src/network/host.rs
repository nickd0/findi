use super::tcp_ping::{tcp_ping, TCP_PING_PORT};
use super::udp_ping::udp_ping;
use super::ping_result::{PingResultOption};

use std::net::{IpAddr};
use std::fmt;

pub type HostVec = Vec<Host>;

#[derive(Copy, Clone)]
pub enum PingType {
  UDP,
  TCP
}

impl fmt::Display for PingType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
    match *self {
      PingType::UDP => { write!(f, "UDP") },
      PingType::TCP => { write!(f, "TCP") }
    }
  }
}

#[derive(Clone)]
pub struct Host {
  pub ip: IpAddr,
  pub ping_res: PingResultOption,
  pub ping_type: Option<PingType>,
  pub tcp_ports: Vec<u16>,
  pub host_name: Option<Result<String, String>>
}

// TODO:
// a  user setting can indicate whether a tcp and/or a udp ping should be use
// also allow for ICMP echo
impl Host {
  pub fn host_ping(ip: IpAddr) -> Host {
    let mut host = Host::new(ip);
    host.ping();
    host
  }

  pub fn new(ip: IpAddr) -> Host {
    Host {
      ip: ip,
      ping_res: None,
      ping_type: None,
      host_name: None,
      tcp_ports: vec![]
    }
  }

  pub fn ping(&mut self) {
    self.ping_res = match udp_ping(self.ip) {
      Ok(t) => {
        self.ping_type = Some(PingType::UDP);
        Some(t)
      },
      Err(_) => {
        match tcp_ping(self.ip) {
          Ok(t) => {
            self.ping_type = Some(PingType::TCP);
            self.tcp_ports.push(TCP_PING_PORT);
            Some(t)
          },
          Err(_) => { None }
        }
      }
    }
  }
}
