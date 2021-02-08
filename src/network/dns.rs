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
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug)]
#[repr(u16)]
pub enum DnsQuestionClass {
    IN = 0x01,
}

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
