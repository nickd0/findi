pub mod ping_result;
pub mod udp_ping;
pub mod tcp_ping;
pub mod host;
pub mod dns;

use pnet::ipnetwork::IpNetwork;
use std::net::Ipv4Addr;

pub fn input_parse(input: &str) -> Result<Vec<Ipv4Addr>, String> {
    if let Ok(IpNetwork::V4(ipn)) = input.parse::<IpNetwork>() {
        Ok(ipn.iter().collect())
    } else {
        Err("Please provide a valid IPv4 CIDR network".to_string())
    }
}
