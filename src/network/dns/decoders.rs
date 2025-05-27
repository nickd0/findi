use super::{serializer, DnsPacket, DnsQuestionClass, DnsQuestionType};

use anyhow::Result;
use bincode::config::Options;
use log::info;
use serde::{Deserialize, Serialize};

pub trait DnsAnswerDecoder {
    fn default_qtype() -> DnsQuestionType;
    fn decode(tx_packet: &DnsPacket, bytes: &[u8]) -> Result<Self>
    where
        Self: std::marker::Sized;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NbnsAnswer {
    name_len: u8,
    qname: [u8; 32],
    qname_term: u8,
    qtype: DnsQuestionType,
    qclass: DnsQuestionClass,
    ttl: u32,
    data_len: u16,
    num_names: u8,
    pub name: [u8; 15],

    #[serde(skip_deserializing)]
    pub hostname: String,
}

impl DnsAnswerDecoder for NbnsAnswer {
    fn default_qtype() -> DnsQuestionType {
        DnsQuestionType::NBSTAT
    }

    fn decode(_: &DnsPacket, bytes: &[u8]) -> Result<NbnsAnswer> {
        info!("Attempting NBNS decode of DNS packet {:?}", bytes);
        let mut ans: NbnsAnswer = serializer().deserialize(&bytes[12..72])?;
        let host_str = String::from_utf8(ans.name.to_vec())?;
        ans.hostname = host_str.trim().to_string();
        Ok(ans)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MdnsAnswer {
    ptr_offset: u16,
    qtype: DnsQuestionType,
    qclass: DnsQuestionClass,
    ttl: u32,
    datalen: u16,

    #[serde(skip_deserializing)]
    pub hostname: String,
}

impl DnsAnswerDecoder for MdnsAnswer {
    fn default_qtype() -> DnsQuestionType {
        DnsQuestionType::PTR
    }

    fn decode(tx_packet: &DnsPacket, bytes: &[u8]) -> Result<Self> {
        info!("Attempting MDNS decode of DNS packet {:?}", bytes);
        let offset = tx_packet.qsizes[0] + 12;

        let mut ret: MdnsAnswer = serializer().deserialize(&bytes[offset..(offset + 12)])?;

        let str_slice: String = bytes[(offset + 13)..((offset + 13) + (ret.datalen - 2) as usize)]
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_nbns_answer_decode() {
        // Query answer:
        // CKAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA!AMACBOOKAIR-CC5Cd05\
        let answer_bytes: [u8; 121] = [
            0xb1, 0x05, 0x84, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x20, 0x43,
            0x4b, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41,
            0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41,
            0x41, 0x41, 0x41, 0x00, 0x00, 0x21, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x41,
            0x01, 0x4d, 0x41, 0x43, 0x42, 0x4f, 0x4f, 0x4b, 0x41, 0x49, 0x52, 0x2d, 0x43, 0x43,
            0x35, 0x43, 0x00, 0x64, 0x00, 0x30, 0x35, 0xad, 0xca, 0xcc, 0x5c, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];

        let tx_packet = DnsPacket::default();

        let nbns_ans = NbnsAnswer::decode(&tx_packet, &answer_bytes).unwrap();

        assert_eq!(nbns_ans.name_len, 32);
        assert_eq!(std::str::from_utf8(&nbns_ans.name), Ok("MACBOOKAIR-CC5C"));
    }

    #[test]
    fn test_mdns_answer_decode() {
        let mdns_answer_bytes: [u8; 80] = [
            0xfe, 0xed, 0x84, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x02, 0x32,
            0x32, 0x01, 0x30, 0x03, 0x31, 0x36, 0x38, 0x03, 0x31, 0x39, 0x32, 0x07, 0x69, 0x6e,
            0x2d, 0x61, 0x64, 0x64, 0x72, 0x04, 0x61, 0x72, 0x70, 0x61, 0x00, 0x00, 0x0c, 0x00,
            0x01, 0xc0, 0x0c, 0x00, 0x0c, 0x00, 0x01, 0x00, 0x00, 0x00, 0x0a, 0x00, 0x19, 0x11,
            0x4e, 0x69, 0x63, 0x6b, 0x44, 0x6f, 0x6e, 0x61, 0x6c, 0x64, 0x2d, 0x69, 0x50, 0x68,
            0x6f, 0x6e, 0x65, 0x05, 0x6c, 0x6f, 0x63, 0x61, 0x6c, 0x00,
        ];

        let mut packet = DnsPacket::default();
        packet.qsizes.push(31);

        let mdns_ans = MdnsAnswer::decode(&packet, &mdns_answer_bytes).unwrap();
        assert_eq!(mdns_ans.hostname, "NickDonald-iPhone.local")
    }
}
