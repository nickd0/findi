// DNS Question/Query implementation.


use std::net::Ipv4Addr;

use serde::{Deserialize, Serialize};
use serde_repr::*;

// DnsQuestion

// A PTR record is used for reverse DNS lookup
// https://www.cloudflare.com/learning/dns/dns-records/dns-ptr-record/
#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug)]
#[repr(u16)]
pub enum DnsQuestionType {
    PTR = 0x0C,
    NBSTAT = 0x21
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug)]
#[repr(u16)]
pub enum DnsQuestionClass {
    IN = 0x01,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DnsQuestion {
    #[serde(skip_serializing)]
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
    ptr_offset: u16,
    qtype: DnsQuestionType,
    qclass: DnsQuestionClass,
    ttl: u32,
    datalen: u16,

    #[serde(skip_deserializing)]
    hostname: String,
}
