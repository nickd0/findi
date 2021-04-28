use std::io::prelude::*;
use std::net::{TcpStream, Ipv4Addr, IpAddr, SocketAddr};
use std::time::{Duration, Instant};
use std::collections::HashSet;

use anyhow::{Result, anyhow};

use super::ping_result::PingResult;

pub const TCP_PING_PORT: u16 = 80;

pub fn parse_portlist(plist_str: &str) -> Result<Vec<u16>> {
    let mut plist: HashSet<u16> = HashSet::new();
    let groups = plist_str.split(',');
    let re = regex::Regex::new(r"(\d+)\s*-\s*(\d+)").unwrap();

    for group in groups {
        match group.parse::<u16>() {
            Ok(port) => { let _ = plist.insert(port); },
            Err(_) => {
                let caps_res = re.captures(group);
                match caps_res {
                    Some(caps) => {
                        // caps[0]
                        let start: u16 = caps[1].parse().unwrap();
                        let end: u16 = caps[2].parse().unwrap();
                        let prange: Vec<u16> = (start..=end).collect();
                        plist.extend(prange);
                    },

                    None => return Err(anyhow!("Could not parse port range"))
                }
            }
        }
    }

    let mut plist_vec = plist.into_iter().collect::<Vec<u16>>();
    plist_vec.sort_unstable();
    Ok(plist_vec)
}

pub fn tcp_ping(ip: Ipv4Addr) -> PingResult {
    tcp_scan_port(&ip, TCP_PING_PORT)
}

pub fn tcp_scan_port(ip: &Ipv4Addr, port: u16) -> PingResult {
    let now = Instant::now();
    let sockaddr = SocketAddr::new(IpAddr::V4(*ip), port);
    // TODO: config
    let to = Some(Duration::from_millis(2000));
    let mut stream = TcpStream::connect_timeout(&sockaddr, to.unwrap())?;

    stream.set_write_timeout(to)?;
    stream.set_read_timeout(to)?;
    stream.write_all(&[1])?;

    Ok(now.elapsed())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_port_range_parse() {
        let valid_range = "22,45,100-102,45";
        let range = parse_portlist(valid_range);
        assert_eq!(range.unwrap(), vec![22, 45, 100, 101, 102]);

        let invalid_range = "22,45,100-";
        let range = parse_portlist(invalid_range);
        assert_eq!(range.is_err(), true);

        let invalid_range1 = "22,45-";
        let range = parse_portlist(invalid_range1);
        assert_eq!(range.is_err(), true);
    }
}
