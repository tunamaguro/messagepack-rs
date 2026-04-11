// Test: encode_with / decode_with custom functions
use messagepack_core::decode::Decode;
use messagepack_core::encode::Encode;
use messagepack_core::io::{IoRead, IoWrite, SliceReader};
use messagepack_derive::{Decode, Encode};

fn encode_doubled<W: IoWrite>(
    val: &u32,
    writer: &mut W,
) -> Result<usize, messagepack_core::encode::Error<W::Error>> {
    let doubled = val * 2;
    messagepack_core::encode::Encode::encode(&doubled, writer)
}

fn decode_halved<'de, R: IoRead<'de>>(
    reader: &mut R,
) -> Result<u32, messagepack_core::decode::Error<R::Error>> {
    let val = <u32 as messagepack_core::decode::DecodeBorrowed<'de>>::decode_borrowed(reader)?;
    Ok(val / 2)
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct Custom {
    #[msgpack(encode_with = "encode_doubled", decode_with = "decode_halved")]
    value: u32,
}

fn main() {
    let c = Custom { value: 10 };
    let mut buf = Vec::new();
    c.encode(&mut buf).unwrap();

    // The encoded value should be 20 (doubled)
    let mut reader = SliceReader::new(&buf);
    let decoded = <Custom as Decode>::decode(&mut reader).unwrap();
    // After decode_halved, it should be 10 again
    assert_eq!(decoded, c);
}
