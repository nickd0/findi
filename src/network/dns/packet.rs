// DNS packet implementation

use super::encoders::{DnsAddressEncoder, DnsPtrEncoder, DnsNbstatEncoder};
use super::query::{DnsAnswer, DnsQuestion, DnsQuestionType};

use bincode::config::{DefaultOptions, Options};
use serde::{Deserialize, Serialize};
use anyhow::Result;

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

#[derive(Serialize, Deserialize, Debug)]
pub struct DnsPacket {
    pub header: DnsPacketHeader,

    #[serde(skip_deserializing, skip_serializing)]
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

    pub fn from(header: DnsPacketHeader) -> DnsPacket {
        Self {
            header,
            ..Default::default()
        }
    }

    #[allow(dead_code)]
    pub fn from_resp_bytes(bytes: &[u8]) -> Result<DnsPacket> {
        let header: DnsPacketHeader = serializer()
            .deserialize(&bytes[..12])?;

        let pack = DnsPacket::from(header);
        Ok(pack)
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
        for q in &self.questions {
            let qbytes = match q.qtype {
                DnsQuestionType::PTR => DnsPtrEncoder::encode(&q),
                DnsQuestionType::NBSTAT => DnsNbstatEncoder::encode(&q),
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

pub fn serializer() -> impl Options {
    DefaultOptions::new()
        .with_fixint_encoding()
        .with_big_endian()
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
    fn test_dns_packet_buffer_parse() {
        let mut packet_buffer: Vec<u8> = PACKET_BYTES.to_vec().clone();
        packet_buffer[7] = 0x01;

        let tid = 0xF0F0;
        let ip = Ipv4Addr::new(10, 0, 9, 10);
        let mut packet = DnsPacket::new(tid, ip);

        let q = DnsQuestion::build_rlookup(ip, DnsQuestionType::PTR);
        packet.add_q(q);
        let _ = packet.as_bytes();

        // A response with hostname a-host.local

        packet_buffer.extend(vec![
            // Name pointer
            0xC0, 0x00,
            // Question type PTR
            0x00, 0x0C,
            // Question class IN
            0x00, 0x01,
            // TTL 600
            0x00, 0x00, 0x02, 0x58,
            // Size of address
            0x00, 0x0e,

            0x06,
            0x61, 0x2d, 0x68, 0x6f, 0x73, 0x74, // "a-host"
            0x05,
            0x6c, 0x6f, 0x63, 0x61, 0x6c, // "local"
            0x00
        ]);

        let resp_packet = DnsPacket::from_resp_bytes(&packet_buffer).unwrap();

        assert_eq!(resp_packet.header.n_answ, 1);
    }
}
