use super::{DnsQuestionClass, DnsQuestionType, serializer};

use bincode;
use bincode::config::{DefaultOptions, Options};
use serde::{Deserialize, Serialize};
use serde_repr::*;

pub trait DnsAnswerDecoder {
    fn decode(bytes: &[u8]) -> Result<Self, String> where Self: std::marker::Sized;
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
    name: [u8;15],
    data: [u8;32],
    data1: [u8;13],
}

impl DnsAnswerDecoder for NbnsAnswer {
    fn decode(bytes: &[u8]) -> Result<NbnsAnswer, String> {
        let ans: NbnsAnswer = serializer().deserialize(bytes)
            .map_err(|e| e.to_string())?;

        Ok(ans)
    }
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
        // CKAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA  ! .     A.MACBOOKPRO-C259 d .\...Y
        let answer_bytes: [u8;105] = [
            0x20, 0x43, 0x4b, 0x41, 0x41, 0x41, 0x41, 0x41,
            0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41,
            0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41,
            0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41,
            0x41, 0x00, 0x00, 0x21, 0x00, 0x01, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x41, 0x01, 0x4d, 0x41, 0x43,
            0x42, 0x4f, 0x4f, 0x4b, 0x50, 0x52, 0x4f, 0x2d,
            0x43, 0x32, 0x35, 0x39, 0x00, 0x64, 0x00, 0xf4,
            0x5c, 0x89, 0xba, 0xc2, 0x59, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00
        ];

        let nbns_ans = NbnsAnswer::decode(&answer_bytes).unwrap();

        assert_eq!(nbns_ans.name_len, 32);
        assert_eq!(std::str::from_utf8(&nbns_ans.name), Ok("MACBOOKPRO-C259"));
    }

}
