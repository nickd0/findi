// use super::{
//     decoders::DnsAnswerDecoder,
//     DnsPacket, DnsQuestionClass,
//     DnsQuestionType,
//     serializer
// };

// use bincode;
// use bincode::config::Options;
// use serde::{Deserialize, Serialize};
// use anyhow::Result;

// #[derive(Serialize, Deserialize, Debug)]
// pub struct MdnsAnswer {
//     ptr_offset: u16,
//     qtype: DnsQuestionType,
//     qclass: DnsQuestionClass,
//     ttl: u32,
//     datalen: u16,

//     #[serde(skip_deserializing)]
//     pub hostname: String,
// }

// impl DnsAnswerDecoder for MdnsAnswer {
//     fn default_qtype() -> DnsQuestionType {
//         DnsQuestionType::PTR
//     }

//     fn decode(tx_packet: &DnsPacket, bytes: &[u8]) -> Result<Self> {

//         let offset = tx_packet.qsizes[0] + 12;

//         let mut ret: MdnsAnswer = serializer()
//             .deserialize(&bytes[offset..(offset + 12)])?;

//         let str_slice: String = bytes[(offset + 13)..((offset + 13) + (ret.datalen - 2) as usize)]
//             .iter()
//             .map(|c| {
//                 if (*c as char).is_ascii_control() {
//                     '.'
//                 } else {
//                     *c as char
//                 }
//             })
//             .collect();
//         ret.hostname = str_slice;
//         Ok(ret)
//     }
// }

// #[cfg(test)]
// mod test {
//     use super::*;


//     #[test]
//     fn test_mdns_answer_decode() {
//        let mdns_answer_bytes: [u8; 80] = [
//             0xfe, 0xed, 0x84, 0x00, 0x00, 0x01, 0x00, 0x01,
//             0x00, 0x00, 0x00, 0x00, 0x02, 0x32, 0x32, 0x01,
//             0x30, 0x03, 0x31, 0x36, 0x38, 0x03, 0x31, 0x39,
//             0x32, 0x07, 0x69, 0x6e, 0x2d, 0x61, 0x64, 0x64,
//             0x72, 0x04, 0x61, 0x72, 0x70, 0x61, 0x00, 0x00,
//             0x0c, 0x00, 0x01, 0xc0, 0x0c, 0x00, 0x0c, 0x00,
//             0x01, 0x00, 0x00, 0x00, 0x0a, 0x00, 0x19, 0x11,
//             0x4e, 0x69, 0x63, 0x6b, 0x44, 0x6f, 0x6e, 0x61,
//             0x6c, 0x64, 0x2d, 0x69, 0x50, 0x68, 0x6f, 0x6e,
//             0x65, 0x05, 0x6c, 0x6f, 0x63, 0x61, 0x6c, 0x00
//        ];

//        let mut packet = DnsPacket::default();
//        packet.qsizes.push(31);

//        let mdns_ans = MdnsAnswer::decode(&packet, &mdns_answer_bytes).unwrap();
//        assert_eq!(mdns_ans.hostname, "NickDonald-iPhone.local")
//     }
// }
