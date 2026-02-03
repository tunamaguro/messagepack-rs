use messagepack_serde::{from_slice, to_vec};
use proptest::prelude::*;

macro_rules! float_eq {
    ($x:expr, $y:expr) => {{
        let is_eq = ($x.is_nan() && $y.is_nan()) || ($x == $y);
        assert!(is_eq);
    }};
}

proptest! {
    #[test]
    fn roundtrip_f32(x in any::<f32>()) {
        let buf = to_vec(&x).unwrap();
        let y:f32 = from_slice(buf.as_slice()).unwrap();

        float_eq!(x, y);
    }
}

proptest! {
    #[test]
    fn roundtrip_f64(x in any::<f64>()) {
        let buf = to_vec(&x).unwrap();
        let y:f64 = from_slice(buf.as_slice()).unwrap();

        float_eq!(x, y);
    }
}
