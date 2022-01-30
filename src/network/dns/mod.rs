/*
TODO:
This is looking good, but one issue is with the bincode serializer.
Strings (and by effect [u8]'s) are serialized to [u8]s and prefaced
with a u64 length of string. Doesn't look like there's anyway around
this except to fork the crate? :(
This will be annoying if we need to suck out those 8 bytes from the
serialized packet

Good note from this r/rust (https://www.reddit.com/r/rust/comments/93x8ej/which_is_better_vecu8_or_u8_for_storage_interface/) post:
> Don't hide the cost of a function.

TODO: Should we use mDNS lookups to the mDNS multicast group and listen for responses
rather then connect to each host individually?
https://stevessmarthomeguide.com/multicast-dns/
All multicast groups are in  224.0.0.0 through 239.255.255.255
mDNS multicast group is on 224.0.0.251
*/

pub mod encoders;
pub mod query;
pub mod packet;
pub mod decodable;

use query::{DnsQuestion, DnsQuestionType, DnsAnswer};
use packet::DnsPacket;

use anyhow::Result;

use std::net::{Ipv4Addr, UdpSocket, ToSocketAddrs};
use std::time::Duration;

// For now, we assume only one answer per reverse lookup, so only return one in this func
pub fn reverse_dns_lookup(ip: Ipv4Addr, qtype: DnsQuestionType) -> Option<DnsAnswer> {
    let port = match qtype {
        DnsQuestionType::Nbstat => 137,
        DnsQuestionType::Ptr => 5353,
        _ => 5353,
    };

    let tid: u16 = 0xF00D;
    let mut packet = DnsPacket::new(tid, ip);
    let nb_q = DnsQuestion::build_rlookup(ip, qtype);
    packet.add_q(nb_q);
    let mut rcv_packets: Vec<DnsPacket> = vec![];

    dns_udp_transact((ip, port), &mut packet, &mut rcv_packets).ok()?;

    // Do we care about the header?
    // let header: DnsPacketHeader = serializer().deserialize(&buf[0..12]);
    // NetBIOS lookups always have the same offset, so no need to parse header for now
    // let answer = NbnsAnswer::decode(&buf[12..])?;
    rcv_packets.pop()?.answers.pop()
}

// TODO make private
pub fn dns_udp_transact<A: ToSocketAddrs>(dst: A, packet: &mut DnsPacket, rcv_packets: &mut Vec<DnsPacket>) -> Result<()> {
    let usock = UdpSocket::bind("0.0.0.0:0")?;
    let mut buf: [u8; 1024] = [0; 1024];
    usock.send_to(&packet.as_bytes().unwrap(), dst)?;
    // TODO: make this timeout configurable
    usock.set_read_timeout(Some(Duration::from_millis(200)))?;

    // Receive all packets from UDP
    while let Ok((sz, _)) = usock.recv_from(&mut buf) {
        if let Ok((packet, _)) = DnsPacket::decode(&buf[0..sz]) {
            rcv_packets.push(packet);
        }
    }
    Ok(())
}
