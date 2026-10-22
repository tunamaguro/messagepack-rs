use messagepack_core::io::{IoWrite, SliceWriter};

#[test]
fn failed_write_keeps_remaining_space() {
    let mut buf = [0_u8; 4];
    let mut writer = SliceWriter::new(&mut buf);

    assert!(writer.write(&[1, 2, 3, 4, 5]).is_err());
    writer.write(&[1, 2, 3, 4]).unwrap();

    assert_eq!(buf, [1, 2, 3, 4]);
}
