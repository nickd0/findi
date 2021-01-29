use std::net::{IpAddr, UdpSocket};
use std::time::{Duration, Instant};
use std::io::{ErrorKind};

use super::ping_result::PingResult;

// TODO: make pinger a trait so that we can use both UdpPinger and IcmpPinger
// with different impl's

// struct UdpPinger {
// }

// impl UdpPinger {

// }

const UDP_PING_PORT: u16 = 39719;

// use the same sockets to connect to every address?
// Async connect and ready after the addresses are printed
// TODO: use tokio once we're using async
// Don't return bool, return a Result
pub fn udp_ping(ip: IpAddr) -> PingResult {
  let sock = UdpSocket::bind("0.0.0.0:34254")?;
  sock.connect((ip, UDP_PING_PORT))?;
  let now = Instant::now();

  sock.send(&[1; 1])?;
  sock.set_read_timeout(Some(Duration::from_millis(400)))?;
  match sock.recv(&mut [0; 1]) {
    Ok(_) => {},
    Err(err) => match err.kind() {
      ErrorKind::WouldBlock => return Err(err),
      _ => {}
    }
  };

  Ok(now.elapsed())
}
