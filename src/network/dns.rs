/*
TODO:
This is looking good, but one issue is with the bincode serializer.
Strings (and by effect [u8]'s) are serialized to [u8]s and prefaced
with a u64 length of string. Doesn't look like there's anyway around
this except to fork the crate? :(
This will be annoying if we need to suck out those 8 bytes from the
serialized packet
*/

use std::net::{Ipv4Addr, IpAddr, UdpSocket};
use serde::{Serialize, Deserialize};
use serde_repr::*;
use bincode::config::{DefaultOptions, Options};
use bincode;
use std::time::Duration;

#[derive(Serialize, Deserialize)]
pub struct DnsPacketHeader {
  trans_id: u16,
  q_flags: u16,
  n_qs: u16,
  n_answ: u16,
  n_auth: u16,
  n_addn: u16
}

impl Default for DnsPacketHeader {
  fn default() -> Self {
    Self {
      trans_id: 0,
      q_flags: 0,
      n_qs: 0,
      n_answ: 0,
      n_auth: 0,
      n_addn: 0
    }
  }
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug)]
#[repr(u16)]
pub enum DnsQuestionType {
  PTR = 0x0C
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug)]
#[repr(u16)]
pub enum DnsQuestionClass {
  IN = 0x01
}

#[derive(Serialize, Deserialize)]
pub struct DnsQuestion {

  #[serde(skip_serializing)]
  addr: Ipv4Addr,

  #[serde(skip_serializing)]
  pub arpa_addr: DnsArpaAddr,

  qtype: DnsQuestionType,
  qclass: DnsQuestionClass
}

#[derive(Serialize, Deserialize)]
pub struct DnsArpaAddr {

  #[serde(skip_serializing)]
  addr: Ipv4Addr,

  addr_enc: Vec<u8>
}

// Use custom serializer?
impl DnsArpaAddr {
  pub fn from(addr: Ipv4Addr) -> DnsArpaAddr {
    let octs = addr.octets();
    let mut addr_str = octs.iter()
        .map(|s| s.to_string()).rev()
        .collect::<Vec<String>>()
        .join(".");

    addr_str.push_str(".in-addr.arpa");

    let mut addr_enc: Vec<u8> = vec![];

    let mut bts: &[u8];
    for chunk in addr_str.split(".") {
      addr_enc.push(chunk.len() as u8);
      bts = chunk.as_bytes();
      addr_enc.extend_from_slice(&mut bts);
    }

    DnsArpaAddr {
      addr,
      addr_enc,
    }
  }
}

// TODO: Serialize to bincode
#[derive(Serialize, Deserialize)]
pub struct DnsPacket {
  header: DnsPacketHeader,
  questions: Vec<DnsQuestion>
}

impl DnsPacket {
  pub fn new(trans_id: u16) -> DnsPacket {
    Self {
      header: DnsPacketHeader {
        trans_id,
        ..Default::default()
      },
      questions: vec![]
    }
  }

  pub fn add_q(&mut self, quest: DnsQuestion) {
    self.header.n_qs += 1;
    self.questions.push(quest);
  }

  pub fn to_bytes(&self) -> Result<Vec<u8>, bincode::Error> {
    let mut bytes: Vec<u8> = vec![];
    // Serialize header
    serializer().serialize_into(&mut bytes, &self.header)?;

    // Special serialize questions
    for q in &self.questions {
      bytes.extend(&q.arpa_addr.addr_enc);
      bytes.push(0x00);
      serializer().serialize_into(&mut bytes, &q)?;
    }

    Ok(bytes)
  }
}

fn serializer() -> impl Options {
  DefaultOptions::new()
    .with_fixint_encoding()
    .with_big_endian()
}

impl DnsQuestion {
  pub fn lookup_ptr(addr: Ipv4Addr) -> DnsQuestion {
    Self::lookup(addr, DnsQuestionType::PTR)
  }

  pub fn lookup(addr: Ipv4Addr, qtype: DnsQuestionType) -> DnsQuestion {
    Self {
      addr,
      arpa_addr: DnsArpaAddr::from(addr),
      qtype,
      qclass: DnsQuestionClass::IN
    }
  }
}

pub fn multicast_dns_lookup(ip: Ipv4Addr) -> Result<String, std::io::Error> {
  let mut packet = DnsPacket::new(0xFEED);
  let dns_q = DnsQuestion::lookup_ptr(ip);
  
  packet.add_q(dns_q);

  let usock = UdpSocket::bind("0.0.0.0:0")?;
  usock.connect((ip, 5353))?;
  usock.send(&packet.to_bytes().unwrap())?;
  usock.set_read_timeout(Some(Duration::from_millis(400)))?;
  // TODO return the host address
  Ok("done".to_string())
}
