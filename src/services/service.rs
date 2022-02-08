// mDNS service implementation.

use crate::state::{
    actions::AppAction,
    store::SharedAppStateStore,
};
use crate::network::dns::{
    packet::DnsPacket,
    query::{DnsQuestion},
    dns_udp_transact,
};

use std::fmt;
use std::thread;
use std::net::{SocketAddr, Ipv4Addr};

pub static MDNS_LOOKUP_ADDR: (Ipv4Addr, u16) = (Ipv4Addr::new(224, 0, 0, 251), 5353);

#[derive(Copy, Clone)]
pub struct Service {
    pub svc_name: &'static str,
    pub subdomain: &'static str,
    pub ip: Option<Ipv4Addr>,
    pub port: Option<u16>,
}

impl Service {
    pub fn new(svc_name: &'static str, subdomain: &'static str) -> Service {
        Service {
            svc_name,
            subdomain,
            ip: None,
            port: None,
        }
    }
}

#[derive(Clone)]
pub struct ServiceDevice {
    pub svc_name: &'static str,
    pub packet: Option<DnsPacket>,
}

impl ServiceDevice {
    pub fn build_from_dns(svc_name: &'static str, packet: DnsPacket) -> ServiceDevice {
        ServiceDevice {
            svc_name,
            packet: Some(packet),
        }
    }
}

impl From<Service> for DnsQuestion {
    fn from(service: Service) -> DnsQuestion {
        DnsQuestion::new(format!("{}._tcp.local", service.subdomain))
    }
}

impl fmt::Display for Service {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "mDNS service {} ({}._tcp.local)", self.svc_name, self.subdomain)
    }
}

pub fn build_service_query_packet(svcs: &[Service]) -> (SocketAddr, DnsPacket) {
    let mut packet = DnsPacket::new(0x01, MDNS_LOOKUP_ADDR.0);
    for svc in svcs {
        packet.add_q((*svc).into())
    }
    (SocketAddr::from(MDNS_LOOKUP_ADDR), packet)
}

pub fn dispatch_service_search(store: SharedAppStateStore, name: &'static str, svcs: &'static [Service]) {
    thread::spawn(move || {
        let (socket_addr, mut packet) = build_service_query_packet(&svcs);
        let mut packets: Vec<DnsPacket> = vec![];
        // TODO: dont expect here
        dns_udp_transact(socket_addr, &mut packet, &mut packets).expect("mDNS query failed");

        let mut lstore = store.lock().unwrap();
        for packet in packets {
            lstore.dispatch(
                AppAction::AddService(
                    ServiceDevice::build_from_dns(name, packet)
                )
            )
        }

    });
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn test_service_question_into() {
        let svc = Service::new("Test service", "_test-service.sub");
        let q: DnsQuestion = svc.into();
        assert_eq!(q.name, "_test-service.sub._tcp.local");
    }

    #[test]
    pub fn test_build_service_query_packet() {
        // Encoded _ipp-tls._tcp.local
        let addr_bts: [u8; 25] = [
            0x08, 0x5f, 0x69, 0x70, 0x70, 0x2d, 0x74, 0x6c,
            0x73, 0x04, 0x5f, 0x74, 0x63, 0x70, 0x05, 0x6c,
            0x6f, 0x63, 0x61, 0x6c, 0x00, 0x00, 0x0c, 0x00,
            0x01
        ];
        let svcs = vec![
            Service::new("Test svc", "_ipp-tls")
        ];
        let (_, mut packet) = build_service_query_packet(&svcs);
        let bts = packet.as_bytes().unwrap();

        // Num questions
        assert_eq!(bts[5], 1);
        assert_eq!(bts[12..37], addr_bts);

    }
}
