use messagepack_core::{Decode as _, Encode as _, io::SliceReader};
use messagepack_derive::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
struct Point {
    x: u32,
    y: u32,
}

#[allow(unused)]
fn encode_point_as_map<__W>(
    point: &Point,
    writer: &mut __W,
) -> Result<usize, messagepack_core::encode::Error<__W::Error>>
where
    __W: messagepack_core::io::IoWrite,
{
    const __FIELD_LEN: usize = 2;
    let mut __size = 0_usize;
    __size += messagepack_core::encode::Encode::encode(
        &messagepack_core::encode::map::MapFormatEncoder(__FIELD_LEN),
        writer,
    )?;

    __size += messagepack_core::encode::Encode::encode(&"x", writer)?;
    __size += messagepack_core::encode::Encode::encode(&point.x, writer)?;
    __size += messagepack_core::encode::Encode::encode(&"y", writer)?;
    __size += messagepack_core::encode::Encode::encode(&point.y, writer)?;

    Ok(__size)
}

#[allow(unused)]
fn encode_point_as_array<__W>(
    point: &Point,
    writer: &mut __W,
) -> Result<usize, messagepack_core::encode::Error<__W::Error>>
where
    __W: messagepack_core::io::IoWrite,
{
    const __FIELD_LEN: usize = 2;
    let mut __size = 0_usize;
    __size += messagepack_core::encode::Encode::encode(
        &messagepack_core::encode::array::ArrayFormatEncoder(__FIELD_LEN),
        writer,
    )?;

    __size += messagepack_core::encode::Encode::encode(&point.x, writer)?;
    __size += messagepack_core::encode::Encode::encode(&point.y, writer)?;

    Ok(__size)
}

#[allow(unused)]
fn decode_point<'__msgpack_de, __R>(
    format: messagepack_core::Format,
    reader: &mut __R,
) -> Result<Point, messagepack_core::decode::Error<__R::Error>>
where
    __R: messagepack_core::io::IoRead<'__msgpack_de>,
{
    enum FormatKind {
        Map(usize),
        Array(usize),
    }
    let kind = match format {
        messagepack_core::Format::FixMap(n) => FormatKind::Map(n.into()),
        messagepack_core::Format::FixArray(n) => FormatKind::Array(n.into()),

        messagepack_core::Format::Array16 => {
            let len = messagepack_core::decode::NbyteReader::<2>::read(reader)?;
            FormatKind::Array(len)
        }
        messagepack_core::Format::Map16 => {
            let len = messagepack_core::decode::NbyteReader::<2>::read(reader)?;
            FormatKind::Map(len)
        }
        messagepack_core::Format::Array32 => {
            let len = messagepack_core::decode::NbyteReader::<4>::read(reader)?;
            FormatKind::Array(len)
        }
        messagepack_core::Format::Map32 => {
            let len = messagepack_core::decode::NbyteReader::<4>::read(reader)?;
            FormatKind::Map(len)
        }
        _ => return Err(messagepack_core::decode::Error::UnexpectedFormat),
    };

    match kind {
        FormatKind::Map(len) => {
            let mut __x = None;
            let mut __y = None;
            for _ in 0..len {
                let key =
                    <messagepack_core::decode::ReferenceStrBinDecoder as messagepack_core::decode::Decode<'__msgpack_de>>::decode(reader)?;

                match key.as_bytes() {
                    b"x" => {
                        if __x.is_some() {
                            return Err(messagepack_core::decode::Error::InvalidData);
                        }
                        __x = Some(
                            <u32 as messagepack_core::decode::Decode<'__msgpack_de>>::decode(
                                reader,
                            )?,
                        );
                    }
                    b"y" => {
                        if __y.is_some() {
                            return Err(messagepack_core::decode::Error::InvalidData);
                        }
                        __y = Some(
                            <u32 as messagepack_core::decode::Decode<'__msgpack_de>>::decode(
                                reader,
                            )?,
                        );
                    }
                    _ => {}
                }
            }

            Ok(Point {
                x: __x.ok_or(messagepack_core::decode::Error::InvalidData)?,
                y: __y.ok_or(messagepack_core::decode::Error::InvalidData)?,
            })
        }
        FormatKind::Array(len) => {
            if len != 2 {
                return Err(messagepack_core::decode::Error::InvalidData);
            }
            let __x = <u32 as messagepack_core::decode::Decode<'__msgpack_de>>::decode(reader)?;
            let __y = <u32 as messagepack_core::decode::Decode<'__msgpack_de>>::decode(reader)?;
            Ok(Point { x: __x, y: __y })
        }
    }
}


#[test]
fn basic_struct() {
    let point = Point { x: 10, y: 20 };

    let mut buf = Vec::new();
    let size = point.encode(&mut buf).unwrap();

    let expected = [
        0x82, // fixmap 2
        0xa1, b'x', // "x"
        0xce, 0x00, 0x00, 0x00, 0x0a, // 10 as u32
        0xa1, b'y', // "y"
        0xce, 0x00, 0x00, 0x00, 0x14, // 20 as u32
    ];
    assert_eq!(buf, expected);

    let mut reader = SliceReader::new(&buf[..size]);
    let decoded = Point::decode(&mut reader).unwrap();
    assert_eq!(decoded, point);
}
