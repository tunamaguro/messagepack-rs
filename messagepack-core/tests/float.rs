use messagepack_core::{
    Decode, Encode,
    io::{SliceReader, SliceWriter},
};
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
        let mut buf = [0; 5];
        let mut writer = SliceWriter::new(&mut buf);
        let written_length = x.encode(&mut writer).unwrap();

        let mut reader = SliceReader::new(&buf[..written_length]);
        let y = f32::decode(&mut reader).unwrap();

        float_eq!(x, y);
    }
}

proptest! {
    #[test]
    fn roundtrip_f64(x in any::<f64>()) {
        let mut buf = [0; 9];
        let mut writer = SliceWriter::new(&mut buf);
        let written_length = x.encode(&mut writer).unwrap();

        let mut reader = SliceReader::new(&buf[..written_length]);
        let y = f64::decode(&mut reader).unwrap();

        float_eq!(x, y);
    }
}
