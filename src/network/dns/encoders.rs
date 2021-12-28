// DNS packet encoders

use super::query::DnsQuestion;

pub trait DnsAddressEncoder {
    fn encode(q: &DnsQuestion) -> Vec<u8>;
}

pub struct DnsPtrEncoder {}

impl DnsAddressEncoder for DnsPtrEncoder {
    fn encode(q: &DnsQuestion) -> Vec<u8> {
        let mut addr_enc: Vec<u8> = vec![];

        let mut bts: &[u8];
        for chunk in q.name.split('.') {
            addr_enc.push(chunk.len() as u8);
            bts = chunk.as_bytes();
            addr_enc.extend_from_slice(&bts);
        };
        addr_enc
    }

    // TODO: move this to a different impl, there should be one for encoding addresses to
    // ARPA searches and one for encoding question host names like in an mDNS serice search
    // fn encode_str(name: &'static str) -> Vec<u8> {
    //     let mut bts: &[u8];
    //     let mut addr_enc: Vec<u8> = vec![];
    //     for chunk in name.split('.') {
    //         addr_enc.push(chunk.len() as u8);
    //         bts = chunk.as_bytes();
    //         addr_enc.extend_from_slice(&bts);
    //     };
    //     addr_enc

    // }
}

pub struct DnsNbstatEncoder {}

impl DnsAddressEncoder for DnsNbstatEncoder {
    fn encode(_: &DnsQuestion) -> Vec<u8> {
        let nb_query: [u8;16] = [b'*', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
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
    use crate::network::dns::query::DnsQuestionType;

    #[test]
    fn test_second_level_encode() {
        let encoded_val = "DBDACODACODJCODBDA";
        let addr = "10.0.9.10";

        assert_eq!(second_level_encode(addr), encoded_val);

        let nb_query: [u8;16] = [('*' as u8), 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

        assert_eq!(
            second_level_encode(std::str::from_utf8(&nb_query).unwrap()),
            "CKAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
        );
    }

    #[test]
    fn test_ptr_encoder() {
        let addr = "10.0.9.10";
        let encoded_addr = "\u{2}10\u{1}9\u{1}0\u{2}10\u{7}in-addr\u{4}arpa";
        let ipv4: Ipv4Addr = addr.parse().unwrap();
        let q = DnsQuestion::build_rlookup(ipv4, DnsQuestionType::PTR);
        assert_eq!(DnsPtrEncoder::encode(&q), encoded_addr.bytes().collect::<Vec<u8>>());
    }
}
