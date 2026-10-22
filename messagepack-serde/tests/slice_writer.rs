use messagepack_core::Format;
use messagepack_serde::ser::{AggressiveMinimize, to_slice_with_config};

#[test]
fn aggressive_minimize_falls_back_to_float_after_buffer_full() {
    let mut buf = [0_u8; 5];
    let len = to_slice_with_config(&1e10_f32, &mut buf, AggressiveMinimize).unwrap();
    assert_eq!(len, 5);
    assert_eq!(buf[0], Format::Float32.as_byte());
}
