// Test: enum should fail to derive
use messagepack_derive::{Encode, Decode};

#[derive(Encode, Decode)]
enum MyEnum {
    A,
    B,
}

fn main() {}
