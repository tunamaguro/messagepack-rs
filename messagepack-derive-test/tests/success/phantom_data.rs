use std::marker::PhantomData;

use messagepack_core::decode::Decode;
use messagepack_core::encode::Encode;
use messagepack_derive::{Decode, Encode};

#[derive(Debug, Encode, Decode)]
struct Foo<T> {
    data: u8,
    marker: PhantomData<T>,
}

struct NonEncodeDecode;

fn assert_derive<'de, T: Encode + Decode<'de>>() {}

fn main() {
    assert_derive::<'_, Foo<u8>>();
    assert_derive::<'_, Foo<NonEncodeDecode>>();
}
