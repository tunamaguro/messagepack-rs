// Test: array mode with key indices
use messagepack_core::decode::Decode;
use messagepack_core::encode::Encode;
use messagepack_derive::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
#[msgpack(array)]
struct Record {
    #[msgpack(key = 0)]
    name: String,
    #[msgpack(key = 1)]
    age: u8,
}

fn assert_derive<'de, T: Encode + Decode<'de>>() {}

fn main() {
    assert_derive::<Record>();
}
