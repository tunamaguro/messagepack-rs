use messagepack_core::decode::Decode;
use messagepack_core::encode::Encode;
use messagepack_derive::{Decode, Encode};

#[derive(Debug, Encode, Decode)]
struct One<T> {
    value: T,
    two: Box<Two<T>>,
}

#[derive(Debug, Encode, Decode)]
struct Two<T> {
    one: Box<One<T>>,
}

fn assert_derive<'de, T: Encode + Decode<'de>>() {}

fn main() {
    assert_derive::<'_, One<u8>>();
    assert_derive::<'_, Two<u8>>();
}
