// DNS Question/Query implementation.

use super::decodable::{serializer, DnsDecodable};
use bincode::Options;

use serde::{Deserialize, Serialize};
use serde_repr::*;
use anyhow::{self, Result};

use std::net::Ipv4Addr;
use std::convert::TryFrom;
use std::str;
use std::iter::Iterator;

// DnsQuestion

// A PTR record is used for reverse DNS lookup
// https://www.cloudflare.com/learning/dns/dns-records/dns-ptr-record/
#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug)]
#[repr(u16)]
pub enum DnsQuestionType {
    PTR = 0x0C,
    ATYPE = 0x01,
    NBSTAT = 0x21
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug)]
#[repr(u16)]
pub enum DnsQuestionClass {
    IN = 0x01,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DnsQuestion {
    #[serde(skip_serializing, skip_deserializing)]
    pub name: String,

    pub qtype: DnsQuestionType,
    pub qclass: DnsQuestionClass,
}

impl DnsQuestion {
    pub fn new(name: String) -> DnsQuestion {
        DnsQuestion {
            name,
            qtype: DnsQuestionType::PTR,
            qclass: DnsQuestionClass::IN,
        }        
    }

    pub fn build_rlookup(ip: Ipv4Addr, qtype: DnsQuestionType) -> DnsQuestion {
        let mut addr_str = ip.octets()
            .iter()
            .map(|s| s.to_string())
            .rev()
            .collect::<Vec<String>>()
            .join(".");

        addr_str.push_str(".in-addr.arpa");

        Self {
            qtype,
            name: addr_str,
            qclass: DnsQuestionClass::IN,
        }
    }
}

// DnsAnswer

#[derive(Serialize, Deserialize, Debug)]
pub struct DnsAnswer {
    #[serde(skip_deserializing, skip_serializing)]
    is_pointer: bool,

    pub ptr_offset: u16,
    pub qtype: DnsQuestionType,
    pub qclass: DnsQuestionClass,
    pub ttl: u32,
    pub datalen: u16,

    #[serde(skip_deserializing, skip_serializing)]
    pub hostname: String,
}

fn decode_ptr_record(bytes: &[u8], terminator: u8) -> Result<(String, usize)> {
    let mut name_buf = vec![];
    let mut idx = 0;
    let mut byte = &bytes[idx];
    while *byte != terminator {
        name_buf.extend(&bytes[(idx + 1)..(idx + 1 + (*byte as usize))]);
        idx = idx + 1 + (*byte as usize);
        byte = &bytes[idx];
        if *byte != terminator {
            name_buf.push('.' as u8);
        }
    }
    let namestr = std::str::from_utf8(&name_buf)?.to_owned();
    Ok((namestr, idx))
}

impl TryFrom<&[u8]> for DnsQuestion {
    type Error = anyhow::Error;

    fn try_from(bytes: &[u8]) -> Result<DnsQuestion> {
        let (name, len) = decode_ptr_record(bytes, 0)?;
        let mut query: DnsQuestion = serializer()
            .deserialize(&bytes[(len + 1)..(len + 5)])?;
        query.name = name;
        Ok(query)
    }
}

impl DnsDecodable<DnsQuestion> for DnsQuestion {
    fn decode(bytes: &[u8]) -> Result<(DnsQuestion, usize)> {
        let (name, len) = decode_ptr_record(bytes, 0)?;
        let mut query: DnsQuestion = serializer()
            .deserialize(&bytes[(len + 1)..(len + 5)])?;
        query.name = name;
        Ok((query, len + 5))
    }
}

impl DnsDecodable<DnsAnswer> for DnsAnswer {
    fn decode(bytes: &[u8]) -> Result<(DnsAnswer, usize)> {
        let mut answ: DnsAnswer = serializer().deserialize(&bytes[..12])?;
        answ.is_pointer = answ.ptr_offset >> 12 == 0xc;
        answ.ptr_offset = answ.ptr_offset & 0xfff;
        println!("datalen: {}", answ.datalen);
        let len = (12 + answ.datalen as usize);
        match answ.qtype {
            DnsQuestionType::PTR => {
                let (name, _) = decode_ptr_record(
                    &bytes[12..len],
                    0xc0
                )?;
                answ.hostname = name;
            },
            _ => {}
        }
        Ok((answ, len))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn test_tryfrom_dns_question_bytes() {
        let bytes: &[u8] = &[
            0x10, 0x5f, 0x73, 0x70, 0x6f, 0x74, 0x69, 0x66,
            0x79, 0x2d, 0x63, 0x6f, 0x6e, 0x6e, 0x65, 0x63,
            0x74, 0x04, 0x5f, 0x74, 0x63, 0x70, 0x05, 0x6c,
            0x6f, 0x63, 0x61, 0x6c, 0x00, 0x00, 0x0c, 0x00,
            0x01
        ];

        let (question, len) = DnsQuestion::decode(bytes).expect("convert failed");

        assert_eq!(question.name, "_spotify-connect._tcp.local");
        assert_eq!(len, 33);
    }

    #[test]
    pub fn test_tryfrom_dns_answer_bytes() {
        let bytes: &[u8] = &[
            0xc0, 0x0c, 0x00, 0x0c, 0x00, 0x01, 0x00, 0x00,
            0x00, 0x0a, 0x00, 0x06, 0x03, 0x4d, 0x42, 0x52,
            0xc0, 0x0c
        ];

        let (answ, len) = DnsAnswer::decode(bytes).expect("convert failed");

        assert_eq!(answ.is_pointer, true);
        assert_eq!(answ.ptr_offset, 0x00c);
        assert_eq!(answ.qtype, DnsQuestionType::PTR);
        assert_eq!(answ.ttl, 10);
        assert_eq!(answ.hostname, "MBR");
        assert_eq!(len, 18);
    }
}
