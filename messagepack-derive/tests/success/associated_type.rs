// Test: associated type encode/decode
use messagepack_core::decode::Decode;
use messagepack_core::encode::Encode;
use messagepack_core::io::SliceReader;
use messagepack_derive::{Decode, Encode};

trait Foo {
    type Item;
}

struct Field<T: Foo> {
    values: Vec<T::Item>,
}

fn main() {
    struct Bar;

    impl Foo for Bar {
        type Item = u8;
    }

    let p = Field::<Bar> {
        values: vec![1, 2, 3],
    };
    let mut buf = Vec::new();
    p.encode(&mut buf).unwrap();

    let mut reader = SliceReader::new(&buf);
    let decoded = <Field<Bar> as Decode>::decode(&mut reader).unwrap();
    assert_eq!(decoded, p);
}
