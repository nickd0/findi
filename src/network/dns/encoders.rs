use std::net::Ipv4Addr;

pub trait DnsAddressEncoder {
    fn encode(ip: &Ipv4Addr) -> Vec<u8>;
}

pub struct DnsPtrEncoder {}

impl DnsAddressEncoder for DnsPtrEncoder {
    fn encode(ip: &Ipv4Addr) -> Vec<u8> {
        let mut addr_str = ip.octets()
            .iter()
            .map(|s| s.to_string())
            .rev()
            .collect::<Vec<String>>()
            .join(".");

        addr_str.push_str(".in-addr.arpa");

        let mut addr_enc: Vec<u8> = vec![];

        let mut bts: &[u8];
        for chunk in addr_str.split(".") {
            addr_enc.push(chunk.len() as u8);
            bts = chunk.as_bytes();
            addr_enc.extend_from_slice(&mut bts);
        };
        addr_enc
    }
}

pub struct DnsNbstatEncoder {}

impl DnsAddressEncoder for DnsNbstatEncoder {
    fn encode(_: &Ipv4Addr) -> Vec<u8> {
        let nb_query: [u8;16] = [('*' as u8), 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut bytes: Vec<u8> = vec![32];
        bytes.extend(
            second_level_encode(&std::str::from_utf8(&nb_query).unwrap()).into_bytes()
        );
        bytes
    }
}


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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_second_level_encode() {
        let encoded_val = "DBDACODACODJCODBDA";
        let addr = "10.0.9.10";

        assert_eq!(second_level_encode(addr), encoded_val);

        let nb_query: [u8;16] = [('*' as u8), 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

        assert_eq!(second_level_encode(std::str::from_utf8(&nb_query).unwrap()), "CKAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA");
    }
}
