// Test: tuple struct encode/decode
use messagepack_core::decode::Decode;
use messagepack_core::encode::Encode;
use messagepack_derive::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
struct Pair(u8, u16);

fn assert_derive<'de, T: Encode + Decode<'de>>() {}

fn main() {
    assert_derive::<Pair>();
}
