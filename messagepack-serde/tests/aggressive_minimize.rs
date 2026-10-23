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
    assert_encoded(&-3e9_f32, Format::Float32, 5);
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
fn chooses_by_encoded_size_at_integer_bounds() {
    assert_encoded(&65535.0_f32, Format::Uint16, 3);
    assert_encoded(&65536.0_f32, Format::Float32, 5);
    assert_encoded(&-32768.0_f32, Format::Int16, 3);
    assert_encoded(&-32769.0_f32, Format::Float32, 5);
    assert_encoded(&-2147483648.0_f32, Format::Float32, 5);
    assert_encoded(&4294967295.0_f64, Format::Uint32, 5);
    assert_encoded(&-2147483648.0_f64, Format::Int32, 5);
    assert_encoded(&4294967296.0_f64, Format::Float32, 5);
    assert_encoded(&18446744073709549568.0_f64, Format::Float64, 9);
}
