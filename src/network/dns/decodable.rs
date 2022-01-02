// Packet decodable trait

use anyhow::Result;
use bincode::config::{DefaultOptions, Options};

pub trait DnsDecodable {
	fn decode(bytes: &[u8]) -> Result<(Self, usize)> where Self: Sized;
}

pub fn serializer() -> impl Options {
    DefaultOptions::new()
        .with_fixint_encoding()
        .with_big_endian()
}
