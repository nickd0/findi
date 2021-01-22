use std::net::{IpAddr, UdpSocket};
use std::time::{Duration, Instant};
use std::io::ErrorKind;

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
pub fn udp_ping(ip: IpAddr) -> bool {
  // TODO: don't use expect
  let sock = UdpSocket::bind("0.0.0.0:34254").expect("Couldn't bind UDP socket");
  // sock.connect(conn_addr);
  sock.connect((ip, UDP_PING_PORT)).expect("Couldn't connect");
  let now = Instant::now();
  sock.send(&[1; 1]).expect("Couldn't send");
  sock.set_read_timeout(Some(Duration::new(0, 100000000))).expect("Couldn't set read timeout");
  match sock.recv(&mut [0; 1]) {
    Ok(_) => println!("No error"),
    Err(err) => match err.kind() {
      ErrorKind::WouldBlock => return false,
      _ => {
        println!("Elapsed time {}", now.elapsed().as_micros());
        return true;
      }
    }
  };
  return true;
}
