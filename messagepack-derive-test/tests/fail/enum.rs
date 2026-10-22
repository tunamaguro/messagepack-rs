// Test: enum should fail to derive
use messagepack_derive::{Decode, Encode};

#[derive(Encode, Decode)]
enum MyEnum {
    A,
    B,
}

fn main() {}
