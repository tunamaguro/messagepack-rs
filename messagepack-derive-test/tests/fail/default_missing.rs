use messagepack_derive::{Decode, Encode};

use messagepack_core::{decode::Decode, encode::Encode};

#[derive(Encode, Decode)]
struct S1<T> {
    #[msgpack(default)]
    foo: T,
}

fn assert_derive<'de, T: Encode + Decode<'de>>() {}

#[derive(Encode, Decode)]
struct S2<T>(#[msgpack(default)] T);


struct NoDefault;

fn main() {
    assert_derive::<S1<NoDefault>>();
    assert_derive::<S2<NoDefault>>();
}
