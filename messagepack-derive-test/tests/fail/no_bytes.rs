use messagepack_core::decode::Decode;
use messagepack_core::encode::Encode;
use messagepack_derive::{Decode, Encode};

struct Foo;

#[derive(Encode, Decode)]
struct S1<'a> {
    #[msgpack(bytes)]
    bytes: &'a [Foo],
}

#[derive(Encode, Decode)]
struct S2 {
    #[msgpack(bytes)]
    bytes: Vec<Foo>,
}

fn assert_derive<'de, T: Encode + Decode<'de>>() {}

fn main() {
    assert_derive::<S1<'_>>();
}
