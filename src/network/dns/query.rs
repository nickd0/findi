// DNS Question/Query implementation.

use super::decodable::{serializer, DnsDecodable};
use bincode::Options;

use serde::{Deserialize, Serialize};
use serde_repr::*;
use anyhow::{self, Result};

use std::net::Ipv4Addr;
use std::str;
use std::iter::Iterator;

// A PTR record is used for reverse DNS lookup
// https://www.cloudflare.com/learning/dns/dns-records/dns-ptr-record/
#[derive(Serialize_repr, Deserialize_repr, Eq, PartialEq, Debug)]
#[repr(u16)]
pub enum DnsQuestionType {
    PTR = 0x0C,
    ATYPE = 0x01,
    NBSTAT = 0x21
}

impl DnsQuestionType {
    fn contains_value(val: u16) -> bool {
        match val {
            x if x == DnsQuestionType::PTR as u16 => true,
            x if x == DnsQuestionType::ATYPE as u16 => true,
            x if x == DnsQuestionType::NBSTAT as u16 => true,
            _ => false
        }
    }
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
    pub ptr_offset: u16,

    pub qtype: DnsQuestionType,
    pub qclass: DnsQuestionClass,
    pub ttl: u32,
    pub datalen: u16,

    #[serde(skip_deserializing, skip_serializing)]
    pub hostname: String,
}

pub struct NbnsAnswer {
    pub query_name: String,
    pub qtype: DnsQuestionType,
    pub qclass: DnsQuestionClass,
    pub ttl: u32,
    pub names: Vec<String>,
    pub hostname: String,
}

fn decode_ptr_record(bytes: &[u8], terminator: u8) -> Result<(String, usize)> {
    let mut name_buf = vec![];
    let mut idx = 0;
    let mut byte = &bytes[idx];
    while *byte != terminator {
        name_buf.extend(&bytes[(idx + 1)..(idx + 1 + (*byte as usize))]);
        idx = idx + 1 + (*byte as usize);
        match &bytes.get(idx) {
            Some(b) => byte = b,
            None => break
        }
        if *byte != terminator && *byte > 0 {
            name_buf.push('.' as u8);
        }
    }
    let namestr = std::str::from_utf8(&name_buf)?.to_owned();
    Ok((namestr, idx))
}

impl DnsDecodable for DnsQuestion {
    fn decode(bytes: &[u8]) -> Result<(DnsQuestion, usize)> {
        let (name, len) = decode_ptr_record(bytes, 0)?;
        let mut query: DnsQuestion = serializer()
            .deserialize(&bytes[(len + 1)..(len + 5)])?;
        query.name = name;
        Ok((query, len + 5))
    }
}

impl DnsDecodable for DnsAnswer {
    fn decode(bytes: &[u8]) -> Result<(DnsAnswer, usize)> {
        let mut name_buf: Vec<u8> = vec![];
        let mut idx = 0;
        let mut chk_byte: u16;
        // build string until we see a valid qtype https://www.rfc-editor.org/rfc/rfc1035
        loop {
            name_buf.push(bytes[idx]);
            chk_byte = u16::from_be_bytes([bytes[idx], bytes[idx + 1]]);
            if DnsQuestionType::contains_value(chk_byte) {
                break
            }
            idx += 1;
        }
        let start = idx + 10;
        let mut answ: DnsAnswer = serializer().deserialize(&bytes[idx..start])?;
        // answ.ptr_offset = answ.ptr_offset & 0xfff;
        let len = start + answ.datalen as usize;
        match answ.qtype {
            DnsQuestionType::PTR => {
                let (name, _) = decode_ptr_record(
                    &bytes[start..len],
                    0xc0
                )?;
                answ.hostname = name;
            },
            DnsQuestionType::NBSTAT => {
                // TODO: more detailed reporting of NBNS queries
                let (nbans, _) = NbnsAnswer::decode(bytes)?;
                answ.hostname = nbans.hostname;
            }
            _ => {}
        }
        Ok((answ, len))
    }
}

impl DnsDecodable for NbnsAnswer {
    fn decode(bytes: &[u8]) -> Result<(NbnsAnswer, usize)> {
        // Get query name string of len bytes[0]
        let qname_len = bytes[0] as usize;
        let query_name = std::str::from_utf8(&bytes[1..qname_len])?.to_owned();

        // Deserialize standard DNS fields
        let dns_fields: DnsAnswer = serializer().deserialize(&bytes[(qname_len + 2)..(qname_len + 12)])?;

        let num_names = bytes[qname_len + 12] as usize;
        let mut names: Vec<String> = vec![];
        let mut hostname: String = "None".to_owned();

        for i in 0..num_names {
            let start = (qname_len + 13) + i * 18;
            let end = qname_len + 13 + (i + 1) * 16;
            let name_bytes = &bytes[start..end];
            match std::str::from_utf8(name_bytes) {
                Ok(name_str) => {
                    names.push(name_str.trim().to_owned());
                    // For now, just use the first name instead of looking at name flags
                    if i == 0 {
                        hostname = name_str.trim().to_owned();
                    }
                },
                Err(_) => {}
            }
        }

        Ok(
            (
                NbnsAnswer {
                    query_name,
                    names,
                    hostname,
                    ttl: dns_fields.ttl,
                    qclass: DnsQuestionClass::IN,
                    qtype: DnsQuestionType::NBSTAT,
                },
                dns_fields.datalen as usize
            )
        )
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

        assert_eq!(answ.ptr_offset, 0x00c);
        assert_eq!(answ.qtype, DnsQuestionType::PTR);
        assert_eq!(answ.ttl, 10);
        assert_eq!(answ.hostname, "MBR");
        assert_eq!(len, 18);
    }

    #[test]
    fn test_nbns_answer_decoder() {

        // Query answer:
        // NJD-SURFACE
        let ans_bytes: &[u8] = &[
            0x20, 0x43, 0x4b, 0x41, 0x41, 0x41, 0x41, 0x41,
            0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41,
            0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41,
            0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41,
            0x41, 0x00, 0x00, 0x21, 0x00, 0x01, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x65, 0x03, 0x4e, 0x4a, 0x44,
            0x2d, 0x53, 0x55, 0x52, 0x46, 0x41, 0x43, 0x45,
            0x20, 0x20, 0x20, 0x20, 0x20, 0x04, 0x00, 0x4e,
            0x4a, 0x44, 0x2d, 0x53, 0x55, 0x52, 0x46, 0x41,
            0x43, 0x45, 0x20, 0x20, 0x20, 0x20, 0x00, 0x04,
            0x00, 0x57, 0x4f, 0x52, 0x4b, 0x47, 0x52, 0x4f,
            0x55, 0x50, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20,
            0x00, 0x84, 0x00, 0xd8, 0xc4, 0x97, 0xec, 0xdb,
            0x6d, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00
        ];

        let (answer, _) = NbnsAnswer::decode(ans_bytes).unwrap();
        assert_eq!(answer.hostname, "NJD-SURFACE");
        assert_eq!(answer.names[2], "WORKGROUP");
    }

}
