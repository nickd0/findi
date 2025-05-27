use super::ping_result::PingResult;
use ::std::io::ErrorKind;
use ::std::net::{Ipv4Addr, UdpSocket};
use ::std::time::{Duration, Instant};
use std::thread::sleep;

use log::info;

const UDP_PING_PORT: u16 = 9989;
const UDP_PING_DUR: Duration = Duration::from_millis(1000);
const UDP_COOL_OFF_MS: i32 = 200;
const UDP_MAX_TRIES: i32 = 3;

pub fn udp_ping(ip: Ipv4Addr) -> PingResult {
    info!("Sending UDP ping to {:?}", ip);
    // TODO make this user settable
    let mut tries = 0;

    let usock = UdpSocket::bind("0.0.0.0:0")?;

    let now = Instant::now();

    while tries >= 0 {
        usock.connect((ip, UDP_PING_PORT))?;
        usock.send(&[1; 1])?;
        usock.set_read_timeout(Some(UDP_PING_DUR))?;
        match usock.recv(&mut [0; 1]) {
            Ok(_) => break,
            Err(err) => match err.kind() {
                ErrorKind::WouldBlock => {
                    if tries == UDP_MAX_TRIES {
                        return Err(err);
                    } else {
                        sleep(Duration::from_millis(
                            ((tries + 1) * UDP_COOL_OFF_MS) as u64,
                        ));
                        tries += 1;
                    }
                }
                _ => break,
            },
        };
    }

    Ok(now.elapsed())
}
