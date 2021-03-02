use super::{
    DnsPacket, DnsQuestion,
    DnsQuestionType,
    decoders::{DnsAnswerDecoder, NbnsAnswer}
};

use anyhow::Result;

use std::{
    time::Duration,
    net::{Ipv4Addr, UdpSocket}
};

// TODO should this return multiple answers or just the first?
pub fn netbios_dns_lookup(ip: Ipv4Addr) -> Result<String> {
    // Pending
    let tid = 0xB105u16;
    let mut packet = DnsPacket::new(tid);
    let nb_q = DnsQuestion::new(ip, DnsQuestionType::NBSTAT);
    let mut buf = [0; 100];
    packet.add_q(nb_q);

    let usock = UdpSocket::bind("0.0.0.0:0")?;
    usock.connect((ip, 137))?;
    usock.send(&packet.to_bytes().unwrap())?;
    usock.set_read_timeout(Some(Duration::from_millis(400)))?;
    usock.recv(&mut buf)?;

    // TODO:
    // - deserialize DNS header from bytes 0..12
    // - deserialize NBNS answer from remaining

    // Do we care about the header?
    // let header: DnsPacketHeader = serializer().deserialize(&buf[0..12]);
    let answer = NbnsAnswer::decode(&buf[12..])?;
    
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
