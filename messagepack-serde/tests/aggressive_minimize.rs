use messagepack_core::Format;
use messagepack_serde::ser::{AggressiveMinimize, to_slice_with_config};
use serde::Serialize;

fn assert_encoded<T: Serialize>(value: &T, format: Format, len: usize) {
    let mut buf = [0_u8; 9];
    let n = to_slice_with_config(value, &mut buf, AggressiveMinimize).unwrap();

    assert_eq!(n, len);
    assert_eq!(buf[0], format.as_byte());
}

#[test]
fn picks_smaller_integer_encoding() {
    assert_encoded(&1.0_f64, Format::PositiveFixInt(1), 1);
}

#[test]
fn picks_smaller_float_encoding() {
    assert_encoded(&1e10_f32, Format::Float32, 5);
    assert_encoded(&1e10_f64, Format::Float32, 5);
}

#[test]
fn keeps_float_when_integer_is_not_smaller() {
    assert_encoded(&((1_u64 << 40) as f64 + 1.0), Format::Float64, 9);
}

#[test]
fn keeps_float_when_integer_does_not_fit() {
    assert_encoded(&1e20_f64, Format::Float64, 9);
}

#[test]
fn picks_integer_or_float_at_bounds() {
    assert_encoded(&65535.0_f32, Format::Uint16, 3);
    assert_encoded(&65536.0_f32, Format::Float32, 5);
    assert_encoded(&4294967295.0_f64, Format::Uint32, 5);
    assert_encoded(&4294967296.0_f64, Format::Float32, 5);
}

#[test]
fn writes_minimized_payload_bytes() {
    let mut buf = [0_u8; 9];
    let n = to_slice_with_config(&-129.0_f64, &mut buf, AggressiveMinimize).unwrap();

    assert_eq!(&buf[..n], &[Format::Int16.as_byte(), 0xff, 0x7f]);
}
