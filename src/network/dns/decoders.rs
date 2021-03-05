use super::{DnsPacket, DnsQuestionClass, DnsQuestionType, serializer};

use bincode;
use bincode::config::Options;
use serde::{Deserialize, Serialize};
use anyhow::Result;

pub trait DnsAnswerDecoder {
    fn decode(tx_packet: &DnsPacket, bytes: &[u8]) -> Result<Self> where Self: std::marker::Sized;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NbnsAnswer {
    name_len: u8,
    qname: [u8;32],
    qname_term: u8,
    qtype: DnsQuestionType,
    qclass: DnsQuestionClass,
    ttl: u32,
    data_len: u16,
    num_names: u8,
    pub name: [u8;15],

    #[serde(skip_deserializing)]
    pub hostname: String
}

impl DnsAnswerDecoder for NbnsAnswer {
    fn decode(_: &DnsPacket, bytes: &[u8]) -> Result<NbnsAnswer> {
        let mut ans: NbnsAnswer = serializer().deserialize(&bytes[12..72])?;
        let host_str = String::from_utf8(ans.name.to_vec())?;
        ans.hostname = host_str.trim().to_string();
        Ok(ans)
    }

    // fn to_string() -> String;
}

// TODO move PTR answer decoding from dns/mod to here
// struct DnsPtrAnswer {}

// impl DnsAnswerDecoder for NbnsAnswer {

// }

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_nbns_answer_decode() {

        // Query answer:
        // CKAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA!AMACBOOKAIR-CC5Cd05\
        let answer_bytes: [u8;121] = [
            0xb1, 0x05, 0x84, 0x00, 0x00, 0x00, 0x00, 0x01,
            0x00, 0x00, 0x00, 0x00, 0x20, 0x43, 0x4b, 0x41,
            0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41,
            0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41,
            0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41,
            0x41, 0x41, 0x41, 0x41, 0x41, 0x00, 0x00, 0x21,
            0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x41,
            0x01, 0x4d, 0x41, 0x43, 0x42, 0x4f, 0x4f, 0x4b,
            0x41, 0x49, 0x52, 0x2d, 0x43, 0x43, 0x35, 0x43,
            0x00, 0x64, 0x00, 0x30, 0x35, 0xad, 0xca, 0xcc,
            0x5c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00
        ];

        let tx_packet = DnsPacket::default();

        let nbns_ans = NbnsAnswer::decode(&tx_packet, &answer_bytes).unwrap();

        assert_eq!(nbns_ans.name_len, 32);
        assert_eq!(std::str::from_utf8(&nbns_ans.name), Ok("MACBOOKAIR-CC5C"));
    }
}
