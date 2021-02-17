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

pub mod netbios;

use bincode;
use bincode::config::{DefaultOptions, Options};
use serde::{Deserialize, Serialize};
use serde_repr::*;
use std::io::{Error, ErrorKind};
use std::net::{Ipv4Addr, UdpSocket};
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

trait DnsAddressEncode {
    fn encode(ip: Ipv4Addr) -> Vec<u8>;
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
        for chunk in addr_str.split(".") {
            addr_enc.push(chunk.len() as u8);
            bts = chunk.as_bytes();
            addr_enc.extend_from_slice(&mut bts);
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

    pub fn from_resp_bytes(quest: &DnsPacket, bytes: &[u8]) -> Result<DnsPacket, String> {
        let header: DnsPacketHeader = serializer()
            .deserialize(&bytes[..12])
            .map_err(|e| e.to_string())?;
        let nans = header.n_answ as usize;
        let mut pack = DnsPacket::from(header);

        let offset = quest.qsizes.iter().fold(0, |sum, i| sum + i) + 12;
        for _ in 0..nans {
            match DnsAnswer::from_bytes(&bytes, offset) {
                Ok(a) => pack.answers.push(a),
                Err(_) => {}
            }
        }

        Ok(pack)
    }

    pub fn add_q(&mut self, quest: DnsQuestion) {
        self.header.n_qs += 1;
        self.questions.push(quest);
    }

    pub fn to_bytes(&mut self) -> Result<Vec<u8>, bincode::Error> {
        let mut bytes: Vec<u8> = vec![];
        // Serialize header
        serializer().serialize_into(&mut bytes, &self.header)?;

        // Special serialize questions
        // Keep track of the question packet sizes
        // so we already know the answer offsets
        for q in &self.questions {
            let qbytes = &q.arpa_addr.addr_enc;
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
    pub fn lookup_ptr(addr: Ipv4Addr) -> DnsQuestion {
        Self::lookup(addr, DnsQuestionType::PTR)
    }

    pub fn lookup(addr: Ipv4Addr, qtype: DnsQuestionType) -> DnsQuestion {
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

impl DnsAnswer {
    pub fn from_bytes(bytes: &[u8], offset: usize) -> Result<DnsAnswer, String> {
        let mut ret: DnsAnswer = serializer()
            .deserialize(&bytes[offset..(offset + 12)])
            .map_err(|e| e.to_string())?;

        let str_slice: String = bytes[(offset + 13)..(13 + offset + (ret.datalen - 2) as usize)]
            .iter()
            .map(|c| {
                if (*c as char).is_ascii_control() {
                    '.'
                } else {
                    *c as char
                }
            })
            .collect();
        ret.hostname = str_slice;
        Ok(ret)
    }
}

pub fn multicast_dns_lookup(ip: Ipv4Addr) -> Result<String, std::io::Error> {
    let mut packet = DnsPacket::new(0xFEED);
    let dns_q = DnsQuestion::lookup_ptr(ip);
    packet.add_q(dns_q);

    let usock = UdpSocket::bind("0.0.0.0:0")?;
    usock.connect((ip, 5353))?;
    usock.send(&packet.to_bytes().unwrap())?;
    usock.set_read_timeout(Some(Duration::from_millis(400)))?;
    let mut buf = [0; 100];
    usock.recv(&mut buf)?;

    // TODO: is there a more efficient way than clone here?
    match DnsPacket::from_resp_bytes(&packet, &buf) {
        Ok(p) => {
            // Ok(p.answers[0].hostname.clone()),
            if let Some(ans) = p.answers.get(0) {
                Ok(ans.hostname.to_owned())
            } else {
                Err(Error::new(ErrorKind::NotFound, "Recieved response with no answers"))
            }
        }
        Err(_) => Err(Error::new(ErrorKind::NotFound, "No hostname returned")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static PACKET_ANS_BYTES: [u8;78] = [
        0xfe, 0xed,

        0x84, 0x0,

        0x0, 0x1,
        0x0, 0x1,
        0x0, 0x0,
        0x0, 0x0,

        0x3, 0x32, 0x31, 0x38, 0x3, 0x31,
        0x32, 0x38, 0x3, 0x31, 0x36, 0x38,
        0x3, 0x31, 0x39, 0x32, 0x7, 0x69,
        0x6e, 0x2d, 0x61, 0x64, 0x64, 0x72,
        0x4, 0x61, 0x72, 0x70, 0x61, 0x0,

        0x0, 0xc,
        0x0, 0x1, 


        0xc0, 0xc,

        0x0, 0xc,

        0x0, 0x1,

        0x0, 0x0, 0x0, 0xa,

        0x0, 0x14,

        0xc,

        0x66, 0x66, 0x2d, 0x63, 0x6f, 0x6d,
        0x2d, 0x33, 0x35, 0x38, 0x36, 0x31,
        0x5, 0x6c, 0x6f, 0x63, 0x61, 0x6c,

        0x0,
    ];

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

    #[test]
    fn test_multicast_dns_packet_build() {
        let tid = 0xF0F0;
        let mut packet = DnsPacket::new(tid);

        assert_eq!(packet.header.trans_id, tid);

        let ip = Ipv4Addr::new(10, 0, 9, 10);
        let q = DnsQuestion::lookup_ptr(ip);

        assert_eq!(q.arpa_addr.addr_enc.iter().map(|c| *c as char).collect::<String>(), "\u{2}10\u{1}9\u{1}0\u{2}10\u{7}in-addr\u{4}arpa");

        packet.add_q(q);

        assert_eq!(packet.header.n_qs, 1);

        assert_eq!(packet.to_bytes().unwrap(), PACKET_BYTES.to_vec());
    }

    #[test]
    fn test_dns_packet_buffer_parse() {
        let mut packet_buffer: Vec<u8> = PACKET_BYTES.to_vec().clone();
        packet_buffer[7] = 0x01;

        let tid = 0xF0F0;
        let mut packet = DnsPacket::new(tid);

        let ip = Ipv4Addr::new(10, 0, 9, 10);
        let q = DnsQuestion::lookup_ptr(ip);
        packet.add_q(q);
        let _ = packet.to_bytes();

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

        assert_eq!(resp_packet.answers.len(), 1);
        assert_eq!(resp_packet.answers[0].hostname, "a-host.local");
    }

    #[test]
    fn test_dns_answer_parse() {
        let ans = DnsAnswer::from_bytes(&PACKET_ANS_BYTES, 46);

        assert_eq!(ans.unwrap().hostname, "ff-com-35861.local")
    }
}
