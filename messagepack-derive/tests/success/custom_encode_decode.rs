// Test: encode_with / decode_with custom functions
use messagepack_core::decode::Decode;
use messagepack_core::encode::Encode;
use messagepack_core::io::{IoRead, IoWrite};
use messagepack_derive::{Decode, Encode};

fn encode_doubled<T, W: IoWrite>(
    _val: &T,
    _writer: &mut W,
) -> Result<usize, messagepack_core::encode::Error<W::Error>> {
    todo!()
}

fn decode_halved<'de, T, R: IoRead<'de>>(
    _reader: &mut R,
) -> Result<T, messagepack_core::decode::Error<R::Error>> {
    todo!()
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct Custom<T> {
    #[msgpack(encode_with = "encode_doubled", decode_with = "decode_halved")]
    value: T,
}

struct NoEncodeDecode;

fn assert_derive<'de, T: Encode + Decode<'de>>() {}

fn main() {
    assert_derive::<Custom<u32>>();
    assert_derive::<Custom<NoEncodeDecode>>();
}
