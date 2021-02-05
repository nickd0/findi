use ipnet::{IpNet, AddrParseError};
use std::net::IpAddr;

// TODO: handle single ip inputs and ranges
// TODO: handle ipv4 and ipv6
// for now, only accept v4 and return a vec of V4
pub fn ip_parse(ip_str: &str) -> Result<Vec<IpAddr>, AddrParseError> {
  match ip_str.parse::<IpAddr>() {
    Ok(host) => return Ok(vec!(host)),
    Err(_) => {}
  };

  let net: IpNet = match ip_str.parse() {
    Ok(n) => n,
    Err(e) => return Err(e)
  };
  return Ok(net.hosts().collect::<Vec<IpAddr>>());
}
