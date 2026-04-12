// Test: explicit map mode on named struct
use messagepack_core::decode::Decode;
use messagepack_core::encode::Encode;
use messagepack_derive::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
#[msgpack(map)]
struct Record {
    name: String,
    age: u8,
}

fn assert_derive<'de, T: Encode + Decode<'de>>() {}

fn main() {
    assert_derive::<Record>();
}
