use messagepack_serde::{from_slice, to_vec};
use proptest::prelude::*;

proptest! {
    #[test]
    fn roundtrip_u8(x in any::<u8>()) {
        let buf = to_vec(&x).unwrap();
        let y = from_slice(buf.as_slice()).unwrap();

        assert_eq!(x, y);
    }
}

proptest! {
    #[test]
    fn roundtrip_u16(x in any::<u16>()) {
        let buf = to_vec(&x).unwrap();
        let y = from_slice(buf.as_slice()).unwrap();

        assert_eq!(x, y);
    }
}

proptest! {
    #[test]
    fn roundtrip_u32(x in any::<u32>()) {
        let buf = to_vec(&x).unwrap();
        let y = from_slice(buf.as_slice()).unwrap();

        assert_eq!(x, y);
    }
}

proptest! {
    #[test]
    fn roundtrip_u64(x in any::<u64>()) {
        let buf = to_vec(&x).unwrap();
        let y = from_slice(buf.as_slice()).unwrap();

        assert_eq!(x, y);
    }
}

proptest! {
    #[test]
    fn roundtrip_usize(x in any::<usize>()) {
        let buf = to_vec(&x).unwrap();
        let y = from_slice(buf.as_slice()).unwrap();

        assert_eq!(x, y);
    }
}

proptest! {
    #[test]
    fn roundtrip_i8(x in any::<i8>()) {
        let buf = to_vec(&x).unwrap();
        let y = from_slice(buf.as_slice()).unwrap();

        assert_eq!(x, y);
    }
}

proptest! {
    #[test]
    fn roundtrip_i16(x in any::<i16>()) {
        let buf = to_vec(&x).unwrap();
        let y = from_slice(buf.as_slice()).unwrap();

        assert_eq!(x, y);
    }
}

proptest! {
    #[test]
    fn roundtrip_i32(x in any::<i32>()) {
        let buf = to_vec(&x).unwrap();
        let y = from_slice(buf.as_slice()).unwrap();

        assert_eq!(x, y);
    }
}

proptest! {
    #[test]
    fn roundtrip_i64(x in any::<i64>()) {
        let buf = to_vec(&x).unwrap();
        let y = from_slice(buf.as_slice()).unwrap();

        assert_eq!(x, y);
    }
}

proptest! {
    #[test]
    fn roundtrip_isize(x in any::<isize>()) {
        let buf = to_vec(&x).unwrap();
        let y = from_slice(buf.as_slice()).unwrap();

        assert_eq!(x, y);
    }
}

proptest! {
    #[test]
    fn roundtrip_u128(x in any::<u128>()) {
        let Ok(buf) = to_vec(&x) else{
            return Ok(());
        };
        let y = from_slice(buf.as_slice()).unwrap();

        assert_eq!(x, y);
    }
}

proptest! {
    #[test]
    fn roundtrip_i128(x in any::<i128>()) {
        let Ok(buf) = to_vec(&x) else{
            return Ok(());
        };
        let y = from_slice(buf.as_slice()).unwrap();

        assert_eq!(x, y);
    }
}
