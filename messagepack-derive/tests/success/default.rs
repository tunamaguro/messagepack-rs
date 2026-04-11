use messagepack_derive::{Decode, Encode};

use messagepack_core::{decode::Decode, encode::Encode};

#[derive(Encode, Decode)]
struct S1<T> {
    #[msgpack(default)]
    foo: Option<T>,
}

fn assert_derive<'de, T: Encode + Decode<'de>>() {}

#[derive(Encode, Decode)]
struct S2<T>(#[msgpack(default)] Option<T>);

fn main() {
    assert_derive::<S1<u8>>();
    assert_derive::<S2<u8>>();
}
