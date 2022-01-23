use super::tcp_ping::{tcp_ping, TCP_PING_PORT};
use super::udp_ping::udp_ping;
use super::ping_result::{PingResultOption};
use super::dns::{
    query::DnsQuestionType,
    reverse_dns_lookup,
};

use anyhow::Result;

use std::net::{Ipv4Addr};
use std::fmt;
use std::collections::HashSet;

pub type HostVec = Vec<Host>;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum PingType {
    UDP,
    TCP
}

impl fmt::Display for PingType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            PingType::UDP => { write!(f, "UDP") },
            PingType::TCP => { write!(f, "TCP") }
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum HostResolutionType {
    MDNS,
    NBNS
}

impl fmt::Display for HostResolutionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            HostResolutionType::MDNS => { write!(f, "Multicast DNS") },
            HostResolutionType::NBNS => { write!(f, "NetBios Name Service") }
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
    pub ping_done: bool
}

// TODO:
// a  user setting can indicate whether a tcp and/or a udp ping should be use
// also allow for ICMP echo
impl Host {
    pub fn host_ping(ip: Ipv4Addr) -> Host {
        let mut host = Host::new(ip);
        host.ping();

        // TODO CONFIG: do multicast lookup in a different thread?
        // Standardize error

        // Perform mDNS reverse lookup
        // Then NBNS NBSTAT query if fails
        match reverse_dns_lookup(ip, DnsQuestionType::PTR) {
            Some(answer) => {
                let hostname = format!("{}", answer);
                host.host_name = Some(Ok(hostname));
                host.res_type = Some(HostResolutionType::MDNS)
            },
            None => {
                match reverse_dns_lookup(ip, DnsQuestionType::NBSTAT) {
                    Some(answer) =>{
                        let hostname = format!("{}", answer);
                        host.host_name = Some(Ok(hostname));
                        host.res_type = Some(HostResolutionType::NBNS)
                    },
                    None => host.host_name = Some(Err("Reverse lookup failed".to_owned()))
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
            res_type: None
        }
    }

    pub fn ping(&mut self) {
        self.ping_res = match udp_ping(self.ip) {
            Ok(t) => {
                self.ping_type = Some(PingType::UDP);
                Some(t)
            },

            Err(_) => {
                match tcp_ping(self.ip) {
                    Ok(t) => {
                        self.ping_type = Some(PingType::TCP);
                        self.tcp_ports.insert(TCP_PING_PORT);
                        Some(t)
                    },
                    Err(_) => None
                }
            }
        }
    }
}
