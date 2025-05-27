use super::dns::{
    decoders::{MdnsAnswer, NbnsAnswer},
    reverse_dns_lookup,
    transactors::UdpTransactorType::{HostTransact, MulticastTransact},
    HostnameLookupUdpPort,
};
use super::ping_result::PingResultOption;
use super::tcp_ping::{tcp_ping, TCP_PING_PORT};
use super::udp_ping::udp_ping;

use anyhow::Result;
use log::warn;

use std::collections::HashSet;
use std::fmt;
use std::net::Ipv4Addr;

pub type HostVec = Vec<Host>;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum PingType {
    UDP,
    TCP,
}

impl fmt::Display for PingType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            PingType::UDP => {
                write!(f, "UDP")
            }
            PingType::TCP => {
                write!(f, "TCP")
            }
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum HostResolutionType {
    MDNS,
    NBNS,
}

impl fmt::Display for HostResolutionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            HostResolutionType::MDNS => {
                write!(f, "Multicast DNS")
            }
            HostResolutionType::NBNS => {
                write!(f, "NetBios Name Service")
            }
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Host {
    pub ip: Ipv4Addr,
    pub ping_res: PingResultOption,
    pub ping_type: Option<PingType>,
    pub tcp_ports: HashSet<u16>,
    pub host_name: Option<Result<String, String>>,
    pub res_type: Option<HostResolutionType>,
    pub ping_done: bool,
}

// TODO:
// a  user setting can indicate whether a tcp and/or a udp ping should be use
// also allow for ICMP echo
impl Host {
    pub fn host_ping(ip: Ipv4Addr) -> Host {
        let mut host = Host::new(ip);
        host.ping();
        if host.ping_res.is_none() {
            host.ping_done = true;
            return host;
        }

        // TODO CONFIG: do multicast lookup in a different thread?
        // Standardize error

        // Perform DNS reverse lookup
        // Then mDNS reverse lookup if fails
        // Then NBNS NBSTAT query if fails
        match reverse_dns_lookup::<MdnsAnswer>(ip, HostnameLookupUdpPort::MDNS, MulticastTransact) {
            Ok(ans) => {
                host.host_name = Some(Ok(ans.hostname));
                host.res_type = Some(HostResolutionType::MDNS)
            }
            Err(_) => {
                match reverse_dns_lookup::<MdnsAnswer>(ip, HostnameLookupUdpPort::DNS, HostTransact)
                {
                    Ok(ans) => {
                        host.host_name = Some(Ok(ans.hostname));
                        host.res_type = Some(HostResolutionType::MDNS)
                    }
                    Err(_) => match reverse_dns_lookup::<NbnsAnswer>(
                        ip,
                        HostnameLookupUdpPort::NBSTAT,
                        HostTransact,
                    ) {
                        Ok(ans) => {
                            host.host_name = Some(Ok(ans.hostname));
                            host.res_type = Some(HostResolutionType::NBNS)
                        }
                        Err(_) => host.host_name = Some(Err("Reverse lookup failed".to_owned())),
                    },
                }
            }
        }

        host.ping_done = true;
        host
    }

    pub fn new(ip: Ipv4Addr) -> Host {
        Host {
            ip,
            ping_res: None,
            ping_type: None,
            host_name: None,
            tcp_ports: HashSet::default(),
            ping_done: false,
            res_type: None,
        }
    }

    pub fn ping(&mut self) {
        self.ping_res = match udp_ping(self.ip) {
            Ok(t) => {
                self.ping_type = Some(PingType::UDP);
                Some(t)
            }

            Err(_) => match tcp_ping(self.ip) {
                Ok(t) => {
                    self.ping_type = Some(PingType::TCP);
                    self.tcp_ports.insert(TCP_PING_PORT);
                    Some(t)
                }
                Err(_) => {
                    warn!("TCP ping failed to {:?}", self.ip);
                    None
                }
            },
        }
    }
}
