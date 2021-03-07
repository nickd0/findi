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
pub mod decoders;

use encoders::DnsAddressEncoder;
use decoders::DnsAnswerDecoder;

use bincode::config::{DefaultOptions, Options};
use serde::{Deserialize, Serialize};
use serde_repr::*;
use anyhow::Result;

use std::net::{Ipv4Addr, UdpSocket, ToSocketAddrs};
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug)]
pub struct DnsPacketHeader {
    trans_id: u16,
    q_flags: u16,
    n_qs: u16,
    n_answ: u16,
    n_auth: u16,
    n_addn: u16,
}

impl Default for DnsPacketHeader {
    fn default() -> Self {
        Self {
            trans_id: 0,
            q_flags: 0,
            n_qs: 0,
            n_answ: 0,
            n_auth: 0,
            n_addn: 0,
        }
    }
}


// A PTR record is used for reverse DNS lookup
// https://www.cloudflare.com/learning/dns/dns-records/dns-ptr-record/
#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug)]
#[repr(u16)]
pub enum DnsQuestionType {
    PTR = 0x0C,
    NBSTAT = 0x21
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug)]
#[repr(u16)]
pub enum DnsQuestionClass {
    IN = 0x01,
}

// TODO use an impl DnsAddressEncode for the encoded addr
// rather than a hardcoded DnsArpaAddr type
#[derive(Serialize, Deserialize, Debug)]
pub struct DnsQuestion {
    #[serde(skip_serializing)]
    addr: Ipv4Addr,

    #[serde(skip_serializing)]
    pub arpa_addr: DnsArpaAddr,

    qtype: DnsQuestionType,
    qclass: DnsQuestionClass,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DnsArpaAddr {
    #[serde(skip_serializing)]
    addr: Ipv4Addr,

    addr_enc: Vec<u8>,
}

// Use custom serializer?
impl DnsArpaAddr {
    pub fn from(addr: Ipv4Addr) -> DnsArpaAddr {
        let octs = addr.octets();
        let mut addr_str = octs
            .iter()
            .map(|s| s.to_string())
            .rev()
            .collect::<Vec<String>>()
            .join(".");

        addr_str.push_str(".in-addr.arpa");

        let mut addr_enc: Vec<u8> = vec![];

        let mut bts: &[u8];
        for chunk in addr_str.split('.') {
            addr_enc.push(chunk.len() as u8);
            bts = chunk.as_bytes();
            addr_enc.extend_from_slice(&bts);
        }

        DnsArpaAddr { addr, addr_enc }
    }
}

// TODO: Serialize to bincode
#[derive(Serialize, Deserialize, Debug)]
pub struct DnsPacket {
    header: DnsPacketHeader,
    questions: Vec<DnsQuestion>,
    qsizes: Vec<usize>,
    answers: Vec<DnsAnswer>,
}

impl Default for DnsPacket {
    fn default() -> DnsPacket {
        Self {
            header: Default::default(),
            questions: vec![],
            qsizes: vec![],
            answers: vec![],
        }
    }
}

impl DnsPacket {
    pub fn new(trans_id: u16) -> DnsPacket {
        Self {
            header: DnsPacketHeader {
                trans_id,
                ..Default::default()
            },
            questions: vec![],
            qsizes: vec![],
            answers: vec![],
        }
    }

    pub fn from(header: DnsPacketHeader) -> DnsPacket {
        Self {
            header,
            ..Default::default()
        }
    }

    pub fn from_resp_bytes(quest: &DnsPacket, bytes: &[u8]) -> Result<DnsPacket> {
        let header: DnsPacketHeader = serializer()
            .deserialize(&bytes[..12])?;

        let pack = DnsPacket::from(header);
        Ok(pack)
    }

    pub fn add_q(&mut self, quest: DnsQuestion) {
        self.header.n_qs += 1;
        self.questions.push(quest);
    }

    pub fn as_bytes(&mut self) -> Result<Vec<u8>> {
        let mut bytes: Vec<u8> = vec![];
        // Serialize header
        serializer().serialize_into(&mut bytes, &self.header)?;

        // Special serialize questions
        // Keep track of the question packet sizes
        // so we already know the answer offsets
        for q in &self.questions {
            let qbytes = match q.qtype {
                DnsQuestionType::PTR => encoders::DnsPtrEncoder::encode(&q.addr),
                DnsQuestionType::NBSTAT => encoders::DnsNbstatEncoder::encode(&q.addr)
            };
            let qlen = qbytes.len();
            bytes.extend(qbytes);
            bytes.push(0x00);
            // Size of encoded address + 4 metadata bytes + terminator byte
            self.qsizes.push(qlen + 5);
            serializer().serialize_into(&mut bytes, &q)?;
        }

        Ok(bytes)
    }
}

fn serializer() -> impl Options {
    DefaultOptions::new()
        .with_fixint_encoding()
        .with_big_endian()
}

impl DnsQuestion {
    pub fn new(addr: Ipv4Addr, qtype: DnsQuestionType) -> DnsQuestion {
        Self {
            addr,
            arpa_addr: DnsArpaAddr::from(addr),
            qtype,
            qclass: DnsQuestionClass::IN,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct DnsAnswer {
    ptr_offset: u16,
    qtype: DnsQuestionType,
    qclass: DnsQuestionClass,
    ttl: u32,
    datalen: u16,

    #[serde(skip_deserializing)]
    hostname: String,
}

// For now, we assume only one answer per reverse lookup, so only return one in this func
pub fn reverse_dns_lookup<T: DnsAnswerDecoder>(ip: Ipv4Addr) -> Result<T> {
    let qtype = T::default_qtype();

    let port = match qtype {
        DnsQuestionType::NBSTAT => 137,
        DnsQuestionType::PTR => 5353
    };

    let tid: u16 = 0xF00D;
    let mut packet = DnsPacket::new(tid);
    let nb_q = DnsQuestion::new(ip, qtype);
    let mut buf = [0; 100];
    packet.add_q(nb_q);

    dns_udp_transact((ip, port), &mut packet, &mut buf)?;

    // Do we care about the header?
    // let header: DnsPacketHeader = serializer().deserialize(&buf[0..12]);
    // NetBIOS lookups always have the same offset, so no need to parse header for now
    // let answer = NbnsAnswer::decode(&buf[12..])?;
    T::decode(&packet, &buf)
}

fn dns_udp_transact<A: ToSocketAddrs>(dst: A, packet: &mut DnsPacket, buf: &mut [u8]) -> Result<()> {
    let usock = UdpSocket::bind("0.0.0.0:0")?;
    usock.connect(dst)?;
    usock.send(&packet.as_bytes().unwrap())?;
    usock.set_read_timeout(Some(Duration::from_millis(400)))?;
    usock.recv(buf)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    static PACKET_BYTES: [u8;40] = [
        // Header //
        // Transaction ID
        0xF0, 0xF0,
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
        0x02, 0x31, 0x30, 0x01,
        0x39, 0x01, 0x30, 0x02,
        0x31, 0x30, 0x07, 0x69,
        0x6e, 0x2d, 0x61, 0x64,
        0x64, 0x72, 0x04, 0x61,
        0x72, 0x70, 0x61, 0x00,

        // Query type "PTR"
        0x00, 0x0C,

        // Query class "IN"
        0x00, 0x01

    ];

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
    fn test_multicast_dns_packet_build() {
        let tid = 0xF0F0;
        let mut packet = DnsPacket::new(tid);

        assert_eq!(packet.header.trans_id, tid);

        let ip = Ipv4Addr::new(10, 0, 9, 10);
        let q = DnsQuestion::new(ip, DnsQuestionType::PTR);

        assert_eq!(q.arpa_addr.addr_enc.iter().map(|c| *c as char).collect::<String>(), "\u{2}10\u{1}9\u{1}0\u{2}10\u{7}in-addr\u{4}arpa");

        packet.add_q(q);

        assert_eq!(packet.header.n_qs, 1);

        assert_eq!(packet.as_bytes().unwrap(), PACKET_BYTES.to_vec());
    }

    #[test]
    fn test_netbios_dns_packet_build() {
        let tid = 0xF00D;

        let mut packet = DnsPacket::new(tid);
        let nb_q = DnsQuestion::new(Ipv4Addr::new(10, 10, 0, 10), DnsQuestionType::NBSTAT);
        packet.add_q(nb_q);

        assert_eq!(packet.as_bytes().unwrap(), NB_PACKET_BYTES);
    }

    #[test]
    fn test_dns_packet_buffer_parse() {
        let mut packet_buffer: Vec<u8> = PACKET_BYTES.to_vec().clone();
        packet_buffer[7] = 0x01;

        let tid = 0xF0F0;
        let mut packet = DnsPacket::new(tid);

        let ip = Ipv4Addr::new(10, 0, 9, 10);
        let q = DnsQuestion::new(ip, DnsQuestionType::PTR);
        packet.add_q(q);
        let _ = packet.as_bytes();

        // A response with hostname a-host.local

        packet_buffer.extend(vec![
            // Name pointer
            0xC0, 0x00,
            // Question type PTR
            0x00, 0x0C,
            // Question class IN
            0x00, 0x01,
            // TTL 600
            0x00, 0x00, 0x02, 0x58,
            // Size of address
            0x00, 0x0e,

            0x06,
            0x61, 0x2d, 0x68, 0x6f, 0x73, 0x74, // "a-host"
            0x05,
            0x6c, 0x6f, 0x63, 0x61, 0x6c, // "local"
            0x00
        ]);

        let resp_packet = DnsPacket::from_resp_bytes(&packet, &packet_buffer).unwrap();

        assert_eq!(resp_packet.header.n_answ, 1);
    }
}
