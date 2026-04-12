use std::marker::PhantomData;

use messagepack_derive::{Decode, Encode};

#[derive(Encode, Decode)]
struct S0<T> {
    #[msgpack(key = 0)]
    value: u8,
    // PhantomData is not allowed with key attribute
    #[msgpack(key = 1)]
    marker: PhantomData<T>,
}

fn main() {}
