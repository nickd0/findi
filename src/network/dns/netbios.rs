use super::{
    DnsPacket, DnsQuestion, DnsAnswer
};

use std::{
    io,
    net::Ipv4Addr
};

// Second level encoding for NetBIOS searches
fn second_level_encode(addr: &str) -> String {
    let mut ret = String::new();
    for c in addr.chars() {

        // Split into two nibbles, add ASCII 'A' and concat
        ret.push((((c as u8) >> 4) + 0x41) as char);
        ret.push((((c as u8) & 0x0f) + 0x41) as char);
    }
    ret
}

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

    #[test]
    fn test_second_level_encode() {
        let encoded_val = "DBDACODACODJCODBDA";
        let addr = "10.0.9.10";

        assert_eq!(second_level_encode(addr), encoded_val);

        let nb_query: [u8;16] = [('*' as u8), 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

        assert_eq!(second_level_encode(std::str::from_utf8(&nb_query).unwrap()), "CKAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA");
    }

        
}
