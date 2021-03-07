// #[cfg(test)]
// mod test {
//     use super::*;
//     use std::{
//         net::Ipv4Addr
//     };

//     static NB_PACKET_BYTES: [u8;50] = [
//         // Header //
//         // Transaction ID
//         0xF0, 0x0D,
//         // Flags
//         0x00,
//         0x00,
//         // Number of questions, answers, authoritative records, additional records
//         0x00, 0x01,
//         0x00, 0x00,
//         0x00, 0x00,
//         0x00, 0x00,

//         // Question //

//         // Address query
//         0x20,
//         0x43, 0x4b, 0x41, 0x41, 0x41,
//         0x41, 0x41, 0x41, 0x41, 0x41,
//         0x41, 0x41, 0x41, 0x41, 0x41,
//         0x41, 0x41, 0x41, 0x41, 0x41,
//         0x41, 0x41, 0x41, 0x41, 0x41,
//         0x41, 0x41, 0x41, 0x41, 0x41,
//         0x41, 0x41,

//         0x00,

//         // Query type "PTR"
//         0x00, 0x21,

//         // Query class "IN"
//         0x00, 0x01

//     ];

//     #[test]
//     fn test_netbios_dns_packet_build() {
//         let tid = 0xF00D;

//         let mut packet = DnsPacket::new(tid);
//         let nb_q = DnsQuestion::new(Ipv4Addr::new(10, 10, 0, 10), DnsQuestionType::NBSTAT);
//         packet.add_q(nb_q);

//         assert_eq!(packet.to_bytes().unwrap(), NB_PACKET_BYTES);
//     }
// }
