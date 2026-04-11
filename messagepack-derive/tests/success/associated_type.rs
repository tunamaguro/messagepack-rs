// Test: associated type encode/decode
use messagepack_core::decode::Decode;
use messagepack_core::encode::Encode;
use messagepack_derive::{Decode, Encode};

trait Foo {
    type Item;
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct Field<T: Foo> {
    values: Vec<T::Item>,
}

fn assert_derive<'de, T: Encode + Decode<'de>>() {}

fn main() {
    #[derive(Debug, PartialEq)]
    struct Bar;

    impl Foo for Bar {
        type Item = u8;
    }

    assert_derive::<'_, Field<Bar>>();
}
