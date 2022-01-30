// DNS Question/Query implementation.

use super::decodable::{serializer, DnsDecodable};
use bincode::Options;

use serde::{Deserialize, Serialize};
use serde_repr::*;
use anyhow::{self, Result};

use std::net::{Ipv4Addr, Ipv6Addr};
use std::convert::TryInto;
use std::fmt;
use std::str;
use std::iter::Iterator;

const PTR_BYTE: u8 = 0xc0;

// A PTR record is used for reverse DNS lookup
// https://www.cloudflare.com/learning/dns/dns-records/dns-ptr-record/
#[derive(Serialize_repr, Deserialize_repr, Eq, PartialEq, Debug, Clone)]
#[repr(u16)]
pub enum DnsQuestionType {
    A = 0x01,
    Ptr = 0x0C,
    Txt = 0x10,
    Aaaa = 0x1C,
    Srv = 0x21,
    // FIXME: this is not compatible with the others
    Nbstat = 0x00,
}


#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Clone)]
#[repr(u16)]
pub enum DnsQuestionClass {
    In = 0x01,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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
            qtype: DnsQuestionType::Ptr,
            qclass: DnsQuestionClass::In,
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
            qclass: DnsQuestionClass::In,
        }
    }
}

// DnsAnswer

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DnsAnswer {
    #[serde(skip_deserializing, skip_serializing)]
    pub ptr_offset: u16,

    #[serde(skip_deserializing, skip_serializing)]
    pub name: String,

    pub qtype: DnsQuestionType,
    pub qclass: DnsQuestionClass,
    pub ttl: u32,
    pub datalen: u16,

    #[serde(skip_deserializing, skip_serializing)]
    pub answer_data: Vec<u8>,
}

// TODO: implement others
impl fmt::Display for DnsAnswer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return match self.qtype {
            DnsQuestionType::Ptr => {
                // TODO: don't unwrap hwere.
                let (rec_str, _) = decode_ptr_record(&self.answer_data, 0xc0).unwrap();
                write!(f, "{}", rec_str)
            },
            DnsQuestionType::A => {
                // TODO: don't unwrap hwere.
                let bytes: [u8; 4] = self.answer_data[..4].try_into().unwrap();
                let addr = Ipv4Addr::from(bytes);
                write!(f, "{}", addr)
            },
            DnsQuestionType::Aaaa => {
                // TODO: don't unwrap hwere.
                let bytes: [u8; 16] = self.answer_data[..].try_into().unwrap();
                let addr = Ipv6Addr::from(bytes);
                write!(f, "{}", addr)
            },
            DnsQuestionType::Txt => {
                let ans_str = str::from_utf8(&self.answer_data[1..]).unwrap();
                write!(f, "{}", ans_str)
            },
            DnsQuestionType::Srv => {
                // TODO: add target with offset calculation
                let priority = ((self.answer_data[0] as u16) << 8) | self.answer_data[1] as u16;
                let weight = ((self.answer_data[2] as u16) << 8) | self.answer_data[3] as u16;
                let port = ((self.answer_data[4] as u16) << 8) | self.answer_data[5] as u16;
                write!(f, "Priority: {}, Weight: {}, Port: {}", priority, weight, port)
            },
            _ => write!(f, "TODO: {:?}", self.qtype)
        }
    }
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
            name_buf.push(b'.');
        }
    }
    let namestr = std::str::from_utf8(&name_buf)?.to_owned();
    Ok((namestr, idx))
}

fn decode_label(bytes: &[u8], start: usize) -> Result<(String, usize)> {
    // Decode a label according to RFC compression rules
    // https://www.rfc-editor.org/rfc/rfc1035#section-4.1.4
    let mut name_buf = vec![];
    let mut byte = bytes[start];
    let mut idx = start;
    let mut count: usize;
    while (byte & PTR_BYTE != PTR_BYTE) && byte != 0x0 {
        name_buf.extend(&bytes[(idx + 1)..(idx + 1 + (byte as usize))]);
        idx = idx + 1 + (byte as usize);
        match bytes.get(idx) {
            Some(b) => {
                byte = *b;
                name_buf.push(b'.');
            },
            None => break
        }
    }
    count = idx - start;
    let mut namestr = std::str::from_utf8(&name_buf)?.to_owned();
    if byte != 0x0 {
        // In this case, we have a pointer label
        let offset = (((bytes[idx] ^ PTR_BYTE) as u16) << 8) | bytes[idx + 1] as u16;
        if let Ok((ptr_name, _)) = decode_label(bytes, offset as usize) {
            namestr.push_str(&ptr_name);
        }
        count += 1;
    }
    count += 1;
    let opt_name = namestr.trim_end_matches('.');
    Ok((opt_name.to_owned(), count))
}

impl DnsDecodable for DnsQuestion {
    fn decode(bytes: &[u8], start: usize) -> Result<(DnsQuestion, usize)> {
        let qbytes = &bytes[start..];
        let (name, len) = decode_ptr_record(&qbytes.to_vec(), 0)?;
        let mut query: DnsQuestion = serializer()
            .deserialize(&qbytes[(len + 1)..(len + 5)])?;
        query.name = name;
        Ok((query, len + 5))
    }
}


impl DnsDecodable for DnsAnswer {
    fn decode(bytes: &[u8], start: usize) -> Result<(DnsAnswer, usize)> {
        let ans_bytes = &bytes[start..];
        let (name, sz) = decode_label(bytes, start).unwrap();
        let start = sz + 10;
        let mut answ: DnsAnswer = serializer().deserialize(&ans_bytes[sz..start])?;
        answ.name = name;
        // answ.ptr_offset = answ.ptr_offset & 0xfff;
        let len = start + answ.datalen as usize;
        match answ.qtype {
            DnsQuestionType::Nbstat => {
                // TODO: more detailed reporting of NBNS queries
                answ.answer_data = ans_bytes.to_vec();
            }
            _ => {
                // TODO: avoid a copy here somehow?
                answ.answer_data = ans_bytes[start..len].to_vec();
            }
        }
        Ok((answ, len))
    }
}

impl DnsDecodable for NbnsAnswer {
    fn decode(bytes: &[u8], start: usize) -> Result<(NbnsAnswer, usize)> {
        // Get query name string of len bytes[0]
        let nbytes = &bytes[start..];
        let qname_len = nbytes[0] as usize;
        let query_name = std::str::from_utf8(&nbytes[1..qname_len])?.to_owned();

        // Deserialize standard DNS fields
        let dns_fields: DnsAnswer = serializer().deserialize(&nbytes[(qname_len + 2)..(qname_len + 12)])?;

        let num_names = nbytes[qname_len + 12] as usize;
        let mut names: Vec<String> = vec![];
        let mut hostname: String = "None".to_owned();

        for i in 0..num_names {
            let start = (qname_len + 13) + i * 18;
            let end = qname_len + 13 + (i + 1) * 16;
            let name_bytes = &nbytes[start..end];
            if let Ok(name_str) = std::str::from_utf8(name_bytes) {
                names.push(name_str.trim().to_owned());
                // For now, just use the first name instead of looking at name flags
                if i == 0 {
                    hostname = name_str.trim().to_owned();
                }
            }
        }

        Ok(
            (
                NbnsAnswer {
                    query_name,
                    names,
                    hostname,
                    ttl: dns_fields.ttl,
                    qclass: DnsQuestionClass::In,
                    qtype: DnsQuestionType::Nbstat,
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

        let (question, len) = DnsQuestion::decode(bytes, 0).expect("convert failed");

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

        let (answ, len) = DnsAnswer::decode(bytes, 0).expect("convert failed");

        assert_eq!(answ.ptr_offset, 0x00c);
        assert_eq!(answ.qtype, DnsQuestionType::Ptr);
        assert_eq!(answ.ttl, 10);
        assert_eq!(format!("{}", answ), "MBR");
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

        let (answer, _) = NbnsAnswer::decode(ans_bytes, 0).unwrap();
        assert_eq!(answer.hostname, "NJD-SURFACE");
        assert_eq!(answer.names[2], "WORKGROUP");
    }

    #[test]
    fn test_tryfrom_dns_addn_answer_bytes() {
        // Test decoding an additional answer

        // A record bytes
        let addn_bytes: &[u8] = &[
            0x01, 0x66, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00,
            0x00, 0x0a, 0x00, 0x04, 0xc0, 0xa8, 0x00, 0x15
        ];

        let (answer, _) = DnsAnswer::decode(addn_bytes, 0).unwrap();
        let expected: Vec<u8> = vec![0xc0, 0xa8, 0x00, 0x15];
        assert_eq!(&answer.name, "f");
        assert_eq!(answer.qtype, DnsQuestionType::A);
        assert_eq!(answer.answer_data, expected);
        assert_eq!(format!("{}", answer), "192.168.0.21");
    }

    #[test]
    fn test_decode_label() {
        let bytes: &[u8] = &[
            0x00, 0x01, 0x84, 0x00, 0x00, 0x01, 0x00, 0x01,
            0x00, 0x00, 0x00, 0x06, 0x08, 0x5f, 0x61, 0x69,
            0x72, 0x70, 0x6f, 0x72, 0x74, 0x04, 0x5f, 0x74,
            0x63, 0x70, 0x05, 0x6c, 0x6f, 0x63, 0x61, 0x6c,
            0x00, 0x00, 0x0c, 0x00, 0x01, 0xc0, 0x0c, 0x00,
            0x0c, 0x00, 0x01, 0x00, 0x00, 0x00, 0x0a, 0x00,
            0x06, 0x03, 0x4d, 0x42, 0x52, 0xc0, 0x0c,
        ];

        let (name, len) = decode_label(bytes, 37).unwrap();
        assert_eq!(name, "_airport._tcp.local");
        assert_eq!(2, len);

        let (name2, len2) = decode_label(bytes, 49).unwrap();
        assert_eq!(name2, "MBR._airport._tcp.local");
        assert_eq!(6, len2);
    }


}
