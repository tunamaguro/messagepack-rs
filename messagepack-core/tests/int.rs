use messagepack_core::{
    Decode, Encode,
    io::{SliceReader, SliceWriter},
};
use proptest::prelude::*;

proptest! {
    #[test]
    fn roundtrip_u8(x in any::<u8>()) {
        let mut buf = [0; 2];
        let mut writer = SliceWriter::new(&mut buf);
        let written_length = x.encode(&mut writer).unwrap();

        let mut reader = SliceReader::new(&buf[..written_length]);
        let y = u8::decode(&mut reader).unwrap();

        assert_eq!(x, y);
    }
}

proptest! {
    #[test]
    fn roundtrip_u16(x in any::<u16>()) {
        let mut buf = [0; 5];
        let mut writer = SliceWriter::new(&mut buf);
        let written_length = x.encode(&mut writer).unwrap();

        let mut reader = SliceReader::new(&buf[..written_length]);
        let y = u16::decode(&mut reader).unwrap();

        assert_eq!(x, y);
    }
}

proptest! {
    #[test]
    fn roundtrip_u32(x in any::<u32>()) {
        let mut buf = [0; 5];
        let mut writer = SliceWriter::new(&mut buf);
        let written_length = x.encode(&mut writer).unwrap();

        let mut reader = SliceReader::new(&buf[..written_length]);
        let y = u32::decode(&mut reader).unwrap();

        assert_eq!(x, y);
    }
}

proptest! {
    #[test]
    fn roundtrip_u64(x in any::<u64>()) {
        let mut buf = [0; 9];
        let mut writer = SliceWriter::new(&mut buf);
        let written_length = x.encode(&mut writer).unwrap();

        let mut reader = SliceReader::new(&buf[..written_length]);
        let y = u64::decode(&mut reader).unwrap();

        assert_eq!(x, y);
    }
}

proptest! {
    #[test]
    fn roundtrip_usize(x in any::<usize>()) {
        let mut buf = [0; 9];
        let mut writer = SliceWriter::new(&mut buf);
        let written_length = x.encode(&mut writer).unwrap();

        let mut reader = SliceReader::new(&buf[..written_length]);
        let y = usize::decode(&mut reader).unwrap();

        assert_eq!(x, y);
    }
}

proptest! {
    #[test]
    fn roundtrip_i8(x in any::<i8>()) {
        let mut buf = [0; 2];
        let mut writer = SliceWriter::new(&mut buf);
        let written_length = x.encode(&mut writer).unwrap();

        let mut reader = SliceReader::new(&buf[..written_length]);
        let y = i8::decode(&mut reader).unwrap();

        assert_eq!(x, y);
    }
}

proptest! {
    #[test]
    fn roundtrip_i16(x in any::<i16>()) {
        let mut buf = [0; 3];
        let mut writer = SliceWriter::new(&mut buf);
        let written_length = x.encode(&mut writer).unwrap();

        let mut reader = SliceReader::new(&buf[..written_length]);
        let y = i16::decode(&mut reader).unwrap();

        assert_eq!(x, y);
    }
}

proptest! {
    #[test]
    fn roundtrip_i32(x in any::<i32>()) {
        let mut buf = [0; 5];
        let mut writer = SliceWriter::new(&mut buf);
        let written_length = x.encode(&mut writer).unwrap();

        let mut reader = SliceReader::new(&buf[..written_length]);
        let y = i32::decode(&mut reader).unwrap();

        assert_eq!(x, y);
    }
}

proptest! {
    #[test]
    fn roundtrip_i64(x in any::<i64>()) {
        let mut buf = [0; 9];
        let mut writer = SliceWriter::new(&mut buf);
        let written_length = x.encode(&mut writer).unwrap();

        let mut reader = SliceReader::new(&buf[..written_length]);
        let y = i64::decode(&mut reader).unwrap();

        assert_eq!(x, y);
    }
}

proptest! {
    #[test]
    fn roundtrip_isize(x in any::<isize>()) {
        let mut buf = [0; 9];
        let mut writer = SliceWriter::new(&mut buf);
        let written_length = x.encode(&mut writer).unwrap();

        let mut reader = SliceReader::new(&buf[..written_length]);
        let y = isize::decode(&mut reader).unwrap();

        assert_eq!(x, y);
    }
}
