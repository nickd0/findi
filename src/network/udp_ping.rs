use ::std::net::{UdpSocket, IpAddr};
use ::std::time::{Duration, Instant};
use ::std::io::ErrorKind;
use std::thread::sleep;
use super::ping_result::PingResult;

const UDP_PING_PORT: u16 = 39719;



pub fn udp_ping(ip: IpAddr) -> PingResult {
  // TODO make this user settable
  let mut tries = 3;

  let usock = UdpSocket::bind("0.0.0.0:0")?;

  let now = Instant::now();

  while tries >= 0 {
    usock.connect((ip, UDP_PING_PORT))?;
    usock.send(&[1; 1])?;
    usock.set_read_timeout(Some(Duration::from_millis(400)))?;
    match usock.recv(&mut [0; 1]) {
      Ok(_) => break,
      Err(err) => match err.kind() {
        ErrorKind::WouldBlock => {
          if tries == 0 {
            return Err(err)
          } else {
            sleep(Duration::from_micros(200));
            tries -= 1;
          }
        },
        _ => break
      }
    };
  }

  Ok(now.elapsed())
}
