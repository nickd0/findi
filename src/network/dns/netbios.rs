use super::{
    DnsPacket, DnsQuestion, DnsAnswer
};

use std::{
    io,
    net::Ipv4Addr
};

pub fn netbios_dns_lookup(ip: Ipv4Addr) -> Result<String, io::Error> {
    // Pending
    Ok("Pending".to_owned())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_netbios_dns_packet_build() {
        let tid = 0xF00D;

        let packet = DnsPacket::new(tid);
    }
}
