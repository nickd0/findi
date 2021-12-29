// Packet decodable trait

use anyhow::Result;
use bincode::config::{DefaultOptions, Options};

pub trait DnsDecodable<T> {
	fn decode(bytes: &[u8]) -> Result<(T, usize)>;
}

pub fn serializer() -> impl Options {
    DefaultOptions::new()
        .with_fixint_encoding()
        .with_big_endian()
}
