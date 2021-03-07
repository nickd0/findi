use std::io::prelude::*;
use std::net::{TcpStream, Ipv4Addr, IpAddr, SocketAddr};
use std::time::{Duration, Instant};

use super::ping_result::PingResult;

pub const TCP_PING_PORT: u16 = 80;

// TODO: custom result type
// TODO: write a post about time::Instant
// Get a result from port 80 and dipslay the HTML title if there is one

pub fn tcp_ping(ip: Ipv4Addr) -> PingResult {
  let now = Instant::now();
  let sockaddr = SocketAddr::new(IpAddr::V4(ip), TCP_PING_PORT);
  let to = Some(Duration::from_millis(2000));
  let mut stream = TcpStream::connect_timeout(&sockaddr, to.unwrap())?;

  stream.set_write_timeout(to)?;
  stream.set_read_timeout(to)?;
  stream.write_all(&[1])?;

  Ok(now.elapsed())
}
