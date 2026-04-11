use messagepack_core::{Decode as _, Encode as _, io::SliceReader};
use messagepack_derive::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
struct Record<'a> {
    /// Expect use `EncodeBytes` and `DecodeBytes` for this field
    #[msgpack(bytes)]
    bytes: &'a [u8],
}

#[test]
fn array_mode() {
    let record = Record {
        bytes: &[1, 2, 3, 4, 5],
    };

    let mut buf = Vec::new();
    let size = record.encode(&mut buf).unwrap();

    let expected = [
        0x81, // fixmap 1
        0xa5, b'b', b'y', b't', b'e', b's', // "bytes"
        0xc4, 0x05, // bin8 with length 5
        1, 2, 3, 4, 5,
    ];
    assert_eq!(buf, expected);

    let mut reader = SliceReader::new(&buf[..size]);
    let decoded = Record::decode(&mut reader).unwrap();
    assert_eq!(decoded, record);
}
