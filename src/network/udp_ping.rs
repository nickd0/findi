use ::std::net::{UdpSocket, IpAddr};
use ::std::time::{Duration, Instant};
use ::std::io::ErrorKind;
use super::ping_result::PingResult;

const UDP_PING_PORT: u16 = 39719;



pub fn udp_ping(ip: IpAddr) -> PingResult {
  let usock = UdpSocket::bind("0.0.0.0:0")?;

  let now = Instant::now();

  usock.connect((ip, UDP_PING_PORT))?;
  usock.send(&[1; 1])?;
  usock.set_read_timeout(Some(Duration::from_millis(400)))?;
  match usock.recv(&mut [0; 1]) {
    Ok(_) => {},
    Err(err) => match err.kind() {
      ErrorKind::WouldBlock => {
        return Err(err)
      },
      _ => {}
    }
  };

  Ok(now.elapsed())
}
