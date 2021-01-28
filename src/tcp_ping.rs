use std::io::prelude::*;
use std::net::{TcpStream, IpAddr, SocketAddr};
use std::time::{Duration, Instant};

const TCP_PING_PORT: u16 = 80;

// TODO: custom result type
// TODO: write a post about time::Instant

pub fn tcp_ping(ip: IpAddr) -> std::io::Result<()> {
  let now = Instant::now();
  let sockaddr = SocketAddr::new(ip, TCP_PING_PORT);
  let to = Some(Duration::from_millis(400));
  let mut stream = TcpStream::connect_timeout(&sockaddr, to.unwrap())?;
  stream.set_write_timeout(to)?;
  stream.set_read_timeout(to)?;
  stream.write(&[1])?;
  println!("Elapsed time {}", now.elapsed().as_millis());
  Ok(())
}
