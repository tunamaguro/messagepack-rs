use messagepack_core::{
    Decode as _, Encode as _,
    io::SliceReader,
    io::{IoRead, IoWrite},
};
use messagepack_derive::{Decode, Encode};

fn encode_doubled<W: IoWrite>(
    val: &u32,
    writer: &mut W,
) -> Result<usize, messagepack_core::encode::Error<W::Error>> {
    (2 * val).encode(writer)
}

fn decode_halved<'de, R: IoRead<'de>>(
    reader: &mut R,
) -> Result<u32, messagepack_core::decode::Error<R::Error>> {
    let doubled = u32::decode(reader)?;
    Ok(doubled / 2)
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct Custom {
    #[msgpack(encode_with = "encode_doubled", decode_with = "decode_halved")]
    value: u32,
}

#[test]
fn custom_encode_decode() {
    let custom = Custom { value: 21 };
    let mut buf = Vec::new();
    let size = custom.encode(&mut buf).unwrap();

    let expected = [
        0x81, // fixmap 1
        0xa5, b'v', b'a', b'l', b'u', b'e', // "value"
        0x2a, // 42 (doubled)
    ];
    assert_eq!(buf, expected);

    let mut reader = SliceReader::new(&buf[..size]);
    let decoded = Custom::decode(&mut reader).unwrap();
    assert_eq!(custom, decoded);
}

fn encode_as_zero<T, W: IoWrite>(
    _val: &T,
    writer: &mut W,
) -> Result<usize, messagepack_core::encode::Error<W::Error>> {
    0u8.encode(writer)
}

fn decode_from_zero<'de, T, R: IoRead<'de>>(
    reader: &mut R,
) -> Result<T, messagepack_core::decode::Error<R::Error>>
where
    T: Default,
{
    let zero = u8::decode(reader)?;
    if zero == 0 {
        Ok(T::default())
    } else {
        Err(messagepack_core::decode::Error::InvalidData)
    }
}

#[derive(Debug, PartialEq, Default)]
struct NoEncodeDecode;

#[derive(Debug, PartialEq, Encode, Decode)]
struct WithNotImplemented<T>
where
    T: Default,
{
    #[msgpack(encode_with = "encode_as_zero", decode_with = "decode_from_zero")]
    value: T,
}

#[test]
fn custom_encode_decode_not_implemented() {
    let with = WithNotImplemented {
        value: NoEncodeDecode,
    };
    let mut buf = Vec::new();
    let size = with.encode(&mut buf).unwrap();

    let expected = [
        0x81, // fixmap 1
        0xa5, b'v', b'a', b'l', b'u', b'e', // "value"
        0x00, // encoded as zero
    ];
    assert_eq!(buf, expected);

    let mut reader = SliceReader::new(&buf[..size]);
    let decoded = WithNotImplemented::decode(&mut reader).unwrap();
    assert_eq!(with, decoded);
}
