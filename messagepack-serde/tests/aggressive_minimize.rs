use messagepack_serde::from_slice;
use messagepack_serde::ser::{AggressiveMinimize, to_slice_with_config};
use proptest::prelude::*;

fn same_f32(left: f32, right: f32) -> bool {
    (left.is_nan() && right.is_nan()) || left == right
}

fn same_f64(left: f64, right: f64) -> bool {
    (left.is_nan() && right.is_nan()) || left == right
}

proptest! {
    #[test]
    fn aggressive_minimize_roundtrips_f32(value in any::<f32>()) {
        let mut encoded = [0_u8; 9];
        let len = to_slice_with_config(&value, &mut encoded, AggressiveMinimize).unwrap();
        let decoded: f32 = from_slice(&encoded[..len]).unwrap();

        prop_assert!(same_f32(value, decoded), "expected {:?}, got {:?}", value, decoded);
    }

    #[test]
    fn aggressive_minimize_roundtrips_f64(value in any::<f64>()) {
        let mut encoded = [0_u8; 9];
        let len = to_slice_with_config(&value, &mut encoded, AggressiveMinimize).unwrap();
        let decoded: f64 = from_slice(&encoded[..len]).unwrap();

        prop_assert!(same_f64(value, decoded), "expected {:?}, got {:?}", value, decoded);
    }
}

macro_rules! int_roundtrip_props {
    ($($name:ident: $ty:ty),+ $(,)?) => {
        proptest! {
            $(
                #[test]
                fn $name(value in any::<$ty>()) {
                    let mut encoded = [0_u8; 9];
                    let len =
                        to_slice_with_config(&value, &mut encoded, AggressiveMinimize).unwrap();
                    let decoded: $ty = from_slice(&encoded[..len]).unwrap();

                    prop_assert_eq!(decoded, value);
                }
            )+
        }
    };
}

int_roundtrip_props! {
    aggressive_minimize_roundtrips_u8: u8,
    aggressive_minimize_roundtrips_u16: u16,
    aggressive_minimize_roundtrips_u32: u32,
    aggressive_minimize_roundtrips_u64: u64,
    aggressive_minimize_roundtrips_i8: i8,
    aggressive_minimize_roundtrips_i16: i16,
    aggressive_minimize_roundtrips_i32: i32,
    aggressive_minimize_roundtrips_i64: i64,
}
