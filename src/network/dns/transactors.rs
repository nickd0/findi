// DNS UDP transactors
use std::net::Ipv4Addr;

pub const UDP_MDNS_MULTICAST_ADDR: Ipv4Addr = Ipv4Addr::new(224, 0, 0, 251);
pub const UDP_MDNS_MULTICAST_PORT: u16 = 5353;

pub enum UdpTransactorType {
    HostTransact,
    MulticastTransact,
}
