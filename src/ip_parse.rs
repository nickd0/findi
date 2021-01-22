use ipnet::{IpNet, AddrParseError};
use std::net::IpAddr;

// TODO: handle single ip inputs and ranges
pub fn ip_parse(ip_str: String) -> Result<Vec<IpAddr>, AddrParseError> {
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
