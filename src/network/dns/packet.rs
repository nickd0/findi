// DNS packet implementation

use super::encoders::{DnsAddressEncoder, DnsPtrEncoder, DnsNbstatEncoder};
use super::query::{DnsAnswer, DnsQuestion, DnsQuestionType};
use super::decodable::{serializer, DnsDecodable};

use bincode::config::Options;
use serde::{Deserialize, Serialize};
use anyhow::{self, Result};

use std::net::Ipv4Addr;

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

pub struct DnsPacket {
    pub header: DnsPacketHeader,

    pub dest_ip: Option<Ipv4Addr>,

    pub questions: Vec<DnsQuestion>,
    pub qsizes: Vec<usize>,
    pub answers: Vec<DnsAnswer>,
}

impl Default for DnsPacket {
    fn default() -> DnsPacket {
        Self {
            header: Default::default(),
            dest_ip: None,
            questions: vec![],
            qsizes: vec![],
            answers: vec![],
        }
    }
}

impl DnsPacket {
    pub fn new(trans_id: u16, dest_ip: Ipv4Addr) -> DnsPacket {
        Self {
            header: DnsPacketHeader {
                trans_id,
                ..Default::default()
            },
            dest_ip: Some(dest_ip),
            questions: vec![],
            qsizes: vec![],
            answers: vec![],
        }
    }

    pub fn add_q(&mut self, quest: DnsQuestion) {
        self.header.n_qs += 1;
        self.questions.push(quest);
    }

    pub fn as_bytes(&mut self) -> Result<Vec<u8>> {
        let mut bytes: Vec<u8> = vec![];
        // Serialize header
        serializer().serialize_into(&mut bytes, &self.header)?;

        // Special serialize questions
        // Keep track of the question packet sizes
        // so we already know the answer offsets
        // TODO: this is being replaced
        for q in &self.questions {
            let qbytes = match q.qtype {
                DnsQuestionType::PTR => DnsPtrEncoder::encode(&q),
                _ => DnsNbstatEncoder::encode(&q),
            };
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

impl DnsPacket {
    pub fn decode(bytes: &[u8]) -> Result<(DnsPacket, usize)> {
        let header: DnsPacketHeader = serializer().deserialize(&bytes[..12])?;
        let mut packet = DnsPacket {
            header,
            ..DnsPacket::default()
        };
        let mut idx = 12;
        for _ in 0..packet.header.n_qs {
            let (q, sz) =  DnsQuestion::decode(&bytes[idx..])?;
            packet.questions.push(q);
            idx += sz;
        }

        for _ in 0..packet.header.n_answ {
            let (a, sz) =  DnsAnswer::decode(&bytes[idx..])?;
            packet.answers.push(a);
            idx += sz;
        }

        Ok((packet, idx))
    }
}


#[cfg(test)]
mod tests {
    use super::*;

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

    static NB_PACKET_BYTES: [u8;50] = [
        // Header //
        // Transaction ID
        0xF0, 0x0D,
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
        0x20,
        0x43, 0x4b, 0x41, 0x41, 0x41,
        0x41, 0x41, 0x41, 0x41, 0x41,
        0x41, 0x41, 0x41, 0x41, 0x41,
        0x41, 0x41, 0x41, 0x41, 0x41,
        0x41, 0x41, 0x41, 0x41, 0x41,
        0x41, 0x41, 0x41, 0x41, 0x41,
        0x41, 0x41,

        0x00,

        // Query type "PTR"
        0x00, 0x21,

        // Query class "IN"
        0x00, 0x01

    ];

    #[test]
    fn test_multicast_dns_packet_build() {
        let tid = 0xF0F0;
        let ip = Ipv4Addr::new(10, 0, 9, 10);
        let mut packet = DnsPacket::new(tid, ip);

        assert_eq!(packet.header.trans_id, tid);

        let q = DnsQuestion::build_rlookup(ip, DnsQuestionType::PTR);

        let enc = DnsPtrEncoder::encode(&q);
        println!("enc: {:?}", enc);
        packet.add_q(q);

        assert_eq!(packet.header.n_qs, 1);

        assert_eq!(packet.as_bytes().unwrap(), PACKET_BYTES.to_vec());
    }

    #[test]
    fn test_netbios_dns_packet_build() {
        let tid = 0xF00D;

        let ip =Ipv4Addr::new(10, 10, 0, 10);
        let mut packet = DnsPacket::new(tid, ip);
        let nb_q = DnsQuestion::build_rlookup(ip, DnsQuestionType::NBSTAT);
        packet.add_q(nb_q);

        assert_eq!(packet.as_bytes().unwrap(), NB_PACKET_BYTES);
    }


    #[test]
    pub fn test_tryfrom_packet_bytes() {
        let bytes: &[u8] = &[0, 1, 132, 0, 0, 0, 0, 0, 0, 0, 0, 6];
        let (packet, _) = DnsPacket::decode(bytes).unwrap();
        let header = packet.header;

        assert_eq!(header.trans_id, 1);
        assert_eq!(header.q_flags, 0x8400);
        assert_eq!(header.n_qs, 0);
        assert_eq!(header.n_answ, 0);
        assert_eq!(header.n_auth, 0);
        assert_eq!(header.n_addn, 6);
    }

    #[test]
    fn test_tryfrom_packet_question_bytes() {
        let bytes: &[u8] = &[
            0x00, 0x01, 0x84, 0x00, 0x00, 0x01, 0x00, 0x01,
            0x00, 0x00, 0x00, 0x06, 0x08, 0x5f, 0x61, 0x69,
            0x72, 0x70, 0x6f, 0x72, 0x74, 0x04, 0x5f, 0x74,
            0x63, 0x70, 0x05, 0x6c, 0x6f, 0x63, 0x61, 0x6c,
            0x00, 0x00, 0x0c, 0x00, 0x01, 0xc0, 0x0c, 0x00,
            0x0c, 0x00, 0x01, 0x00, 0x00, 0x00, 0x0a, 0x00,
            0x06, 0x03, 0x4d, 0x42, 0x52, 0xc0, 0x0c, 0xc0,
            0x31, 0x00, 0x21, 0x00, 0x01, 0x00, 0x00, 0x00,
            0x0a, 0x00, 0x0c, 0x00, 0x00, 0x00, 0x00, 0x13,
            0x91, 0x03, 0x4d, 0x42, 0x52, 0xc0, 0x1a, 0xc0,
            0x31, 0x00, 0x10, 0x00, 0x01, 0x00, 0x00, 0x00,
            0x0a, 0x00, 0xa9, 0xa8, 0x77, 0x61, 0x4d, 0x41,
            0x3d, 0x36, 0x34, 0x2d, 0x41, 0x35, 0x2d, 0x43,
            0x33, 0x2d, 0x35, 0x46, 0x2d, 0x32, 0x31, 0x2d,
            0x31, 0x31, 0x2c, 0x72, 0x61, 0x4d, 0x41, 0x3d,
            0x36, 0x34, 0x2d, 0x41, 0x35, 0x2d, 0x43, 0x33,
            0x2d, 0x36, 0x41, 0x2d, 0x43, 0x32, 0x2d, 0x46,
            0x37, 0x2c, 0x72, 0x61, 0x4d, 0x32, 0x3d, 0x36,
            0x34, 0x2d, 0x41, 0x35, 0x2d, 0x43, 0x33, 0x2d,
            0x36, 0x41, 0x2d, 0x43, 0x32, 0x2d, 0x46, 0x36,
            0x2c, 0x72, 0x61, 0x4e, 0x6d, 0x3d, 0x64, 0x6f,
            0x75, 0x62, 0x6c, 0x65, 0x62, 0x75, 0x62, 0x62,
            0x6c, 0x65, 0x2c, 0x72, 0x61, 0x43, 0x68, 0x3d,
            0x33, 0x36, 0x2c, 0x72, 0x43, 0x68, 0x32, 0x3d,
            0x36, 0x2c, 0x72, 0x61, 0x53, 0x74, 0x3d, 0x30,
            0x2c, 0x72, 0x61, 0x4e, 0x41, 0x3d, 0x30, 0x2c,
            0x73, 0x79, 0x46, 0x6c, 0x3d, 0x30, 0x78, 0x38,
            0x41, 0x30, 0x43, 0x2c, 0x73, 0x79, 0x41, 0x50,
            0x3d, 0x31, 0x32, 0x30, 0x2c, 0x73, 0x79, 0x56,
            0x73, 0x3d, 0x37, 0x2e, 0x39, 0x2e, 0x31, 0x2c,
            0x73, 0x72, 0x63, 0x76, 0x3d, 0x37, 0x39, 0x31,
            0x30, 0x30, 0x2e, 0x32, 0x2c, 0x62, 0x6a, 0x53,
            0x64, 0x3d, 0x33, 0x33, 0x03, 0x4d, 0x42, 0x52,
            0x0c, 0x5f, 0x64, 0x65, 0x76, 0x69, 0x63, 0x65,
            0x2d, 0x69, 0x6e, 0x66, 0x6f, 0xc0, 0x15, 0x00,
            0x10, 0x00, 0x01, 0x00, 0x00, 0x00, 0x0a, 0x00,
            0x13, 0x12, 0x6d, 0x6f, 0x64, 0x65, 0x6c, 0x3d,
            0x41, 0x69, 0x72, 0x50, 0x6f, 0x72, 0x74, 0x37,
            0x2c, 0x31, 0x32, 0x30, 0xc0, 0x49, 0x00, 0x1c,
            0x00, 0x01, 0x00, 0x00, 0x00, 0x0a, 0x00, 0x10,
            0xfe, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x66, 0xa5, 0xc3, 0xff, 0xfe, 0x5f, 0x21, 0x11,
            0xc0, 0x49, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00,
            0x00, 0x0a, 0x00, 0x04, 0xc0, 0xa8, 0x00, 0x12,
            0xc0, 0x49, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00,
            0x00, 0x0a, 0x00, 0x04, 0xa9, 0xfe, 0x7d, 0x3d
        ];

        let (packet, _) = DnsPacket::decode(bytes).expect("decode failed");

        assert_eq!(packet.header.n_qs, 1);
        assert_eq!(packet.header.n_answ, 1);

        let q = &packet.questions[0];
        assert_eq!(q.name, "_airport._tcp.local");
        assert_eq!(q.qtype, DnsQuestionType::PTR);

        let a = &packet.answers[0];
        assert_eq!(a.hostname, "MBR");
        assert_eq!(a.qtype, DnsQuestionType::PTR);
    }
}
