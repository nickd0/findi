use super::{
    DnsPacket, DnsQuestion,
    DnsQuestionType,
    decoders::{DnsAnswerDecoder, NbnsAnswer},
    dns_udp_transact
};

use anyhow::Result;

use std::{
    net::Ipv4Addr
};

// TODO should this return multiple answers or just the first?
pub fn netbios_dns_lookup(ip: Ipv4Addr) -> Result<String> {
    let tid: u16 = 0xB105;
    let mut packet = DnsPacket::new(tid);
    let nb_q = DnsQuestion::new(ip, DnsQuestionType::NBSTAT);
    let mut buf = [0; 100];
    packet.add_q(nb_q);

    dns_udp_transact((ip, 137), &mut packet, &mut buf)?;

    // Do we care about the header?
    // let header: DnsPacketHeader = serializer().deserialize(&buf[0..12]);
    // NetBIOS lookups always have the same offset, so no need to parse header for now
    let answer = NbnsAnswer::decode(&packet, &buf[12..])?;
    
    let host_str = String::from_utf8(answer.name.to_vec())?;
    Ok(host_str.trim().to_string())
}

#[cfg(test)]
mod test {
    use super::*;

    static NB_PACKET_BYTES: [u8;50] = [
        // Header //
        // Transaction ID
        0xF0, 0x0D,
        // Flags
        0x00,
        0x00,
        // Number of questions, answers, authoritative records, additional records
        0x00, 0x01,
        0x00, 0x00,
        0x00, 0x00,
        0x00, 0x00,

        // Question //

        // Address query
        0x20,
        0x43, 0x4b, 0x41, 0x41, 0x41,
        0x41, 0x41, 0x41, 0x41, 0x41,
        0x41, 0x41, 0x41, 0x41, 0x41,
        0x41, 0x41, 0x41, 0x41, 0x41,
        0x41, 0x41, 0x41, 0x41, 0x41,
        0x41, 0x41, 0x41, 0x41, 0x41,
        0x41, 0x41,

        0x00,

        // Query type "PTR"
        0x00, 0x21,

        // Query class "IN"
        0x00, 0x01

    ];

    #[test]
    fn test_netbios_dns_packet_build() {
        let tid = 0xF00D;

        let mut packet = DnsPacket::new(tid);
        let nb_q = DnsQuestion::new(Ipv4Addr::new(10, 10, 0, 10), DnsQuestionType::NBSTAT);
        packet.add_q(nb_q);

        assert_eq!(packet.to_bytes().unwrap(), NB_PACKET_BYTES);
    }
}
