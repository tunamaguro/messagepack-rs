/// Tests for potential issues found during review.
use messagepack_core::decode::{Decode, Error};
use messagepack_core::encode::Encode;
use messagepack_core::io::SliceReader;
use messagepack_derive::{Decode, Encode};

// ================================================================
// Issue: Option<T> fields treated differently in map vs array decode
//
// Map decode allows missing Option<T> fields (defaults to None).
// Array decode of the same named struct should behave the same:
// a shorter array should fill trailing Option<T> fields with None.
// ================================================================

#[derive(Debug, PartialEq, Encode, Decode)]
struct WithOption {
    foo: u8,
    bar: Option<u8>,
}

/// Map decode with missing Option field works (baseline).
#[test]
fn option_missing_from_map_decode() {
    // {"foo": 12}
    let data = [0x81, 0xa3, b'f', b'o', b'o', 0x0c];
    let mut reader = SliceReader::new(&data);
    let decoded = <WithOption as Decode>::decode(&mut reader).unwrap();
    assert_eq!(
        decoded,
        WithOption {
            foo: 12,
            bar: None
        }
    );
}

/// Array decode with missing trailing Option field should also default to None.
/// This mirrors the behavior of map decode where optional fields can be absent.
#[test]
fn option_missing_from_array_decode() {
    // fixarray(1), 12  — only foo, bar is missing
    let data = [0x91, 0x0c];
    let mut reader = SliceReader::new(&data);
    let result = <WithOption as Decode>::decode(&mut reader);
    // If this is Err, Option<T> fields in array decode are not treated as optional
    // (inconsistent with map decode behavior).
    match result {
        Ok(decoded) => {
            assert_eq!(
                decoded,
                WithOption {
                    foo: 12,
                    bar: None
                }
            );
        }
        Err(Error::InvalidData) => {
            panic!(
                "BUG: array decode rejects shorter array even though trailing field is Option<T>. \
                 Map decode would accept this and set bar = None."
            );
        }
        Err(e) => {
            panic!("Unexpected error: {:?}", e);
        }
    }
}

// ================================================================
// Roundtrip: Option<T> present with value through both formats
// ================================================================

#[test]
fn option_present_roundtrip() {
    let value = WithOption {
        foo: 12,
        bar: Some(42),
    };
    let mut buf = Vec::new();
    value.encode(&mut buf).unwrap();

    let mut reader = SliceReader::new(&buf);
    let decoded = <WithOption as Decode>::decode(&mut reader).unwrap();
    assert_eq!(decoded, value);
}

#[test]
fn option_none_roundtrip() {
    let value = WithOption {
        foo: 12,
        bar: None,
    };
    let mut buf = Vec::new();
    value.encode(&mut buf).unwrap();

    let mut reader = SliceReader::new(&buf);
    let decoded = <WithOption as Decode>::decode(&mut reader).unwrap();
    assert_eq!(decoded, value);
}

// ================================================================
// Verify Option<T> + bytes works correctly
// ================================================================

#[derive(Debug, PartialEq, Encode, Decode)]
struct WithOptionalBytes<'a> {
    name: &'a str,
    #[msgpack(bytes)]
    data: Option<&'a [u8]>,
}

#[test]
fn option_bytes_present_roundtrip() {
    let value = WithOptionalBytes {
        name: "test",
        data: Some(&[1, 2, 3]),
    };
    let mut buf = Vec::new();
    value.encode(&mut buf).unwrap();

    let mut reader = SliceReader::new(&buf);
    let decoded = <WithOptionalBytes as Decode>::decode(&mut reader).unwrap();
    assert_eq!(decoded, value);
}

#[test]
fn option_bytes_none_roundtrip() {
    let value = WithOptionalBytes {
        name: "test",
        data: None,
    };
    let mut buf = Vec::new();
    value.encode(&mut buf).unwrap();

    let mut reader = SliceReader::new(&buf);
    let decoded = <WithOptionalBytes as Decode>::decode(&mut reader).unwrap();
    assert_eq!(decoded, value);
}

// ================================================================
// Verify Box<T> decode works
// ================================================================

#[derive(Debug, PartialEq, Encode, Decode)]
struct WithBox {
    value: Box<u32>,
}

#[test]
fn box_field_roundtrip() {
    let value = WithBox {
        value: Box::new(42),
    };
    let mut buf = Vec::new();
    value.encode(&mut buf).unwrap();

    let mut reader = SliceReader::new(&buf);
    let decoded = <WithBox as Decode>::decode(&mut reader).unwrap();
    assert_eq!(decoded, value);
}

// ================================================================
// Verify Option<Box<T>> decode works
// ================================================================

#[derive(Debug, PartialEq, Encode, Decode)]
struct WithOptionBox {
    value: Option<Box<u32>>,
}

#[test]
fn option_box_some_roundtrip() {
    let value = WithOptionBox {
        value: Some(Box::new(42)),
    };
    let mut buf = Vec::new();
    value.encode(&mut buf).unwrap();

    let mut reader = SliceReader::new(&buf);
    let decoded = <WithOptionBox as Decode>::decode(&mut reader).unwrap();
    assert_eq!(decoded, value);
}

#[test]
fn option_box_none_roundtrip() {
    let value = WithOptionBox { value: None };
    let mut buf = Vec::new();
    value.encode(&mut buf).unwrap();

    let mut reader = SliceReader::new(&buf);
    let decoded = <WithOptionBox as Decode>::decode(&mut reader).unwrap();
    assert_eq!(decoded, value);
}
