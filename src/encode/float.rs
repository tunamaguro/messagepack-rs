use super::{Encode, Error, Result};
use crate::formats::Format;

impl Encode for f32 {
    fn encode<T>(&self, buf: &mut T) -> Result<usize>
    where
        T: Extend<u8>,
    {
        let it = Format::Float32.into_iter().chain(self.to_be_bytes());
        buf.extend(it);
        Ok(5)
    }
    fn encode_to_iter_mut<'a>(&self, buf: &mut impl Iterator<Item = &'a mut u8>) -> Result<usize> {
        const SIZE: usize = 5;
        let it = &mut Format::Float32.into_iter().chain(self.to_be_bytes());
        for (byte, to) in it.zip(buf) {
            *to = byte
        }
        if it.next().is_none() {
            Ok(SIZE)
        } else {
            Err(Error::BufferFull)
        }
    }
}

impl Encode for f64 {
    fn encode<T>(&self, buf: &mut T) -> Result<usize>
    where
        T: Extend<u8>,
    {
        let it = Format::Float64.into_iter().chain(self.to_be_bytes());
        buf.extend(it);
        Ok(9)
    }
    fn encode_to_iter_mut<'a>(&self, buf: &mut impl Iterator<Item = &'a mut u8>) -> Result<usize> {
        const SIZE: usize = 9;
        let it = &mut Format::Float64.into_iter().chain(self.to_be_bytes());
        for (byte, to) in it.zip(buf) {
            *to = byte
        }
        if it.next().is_none() {
            Ok(SIZE)
        } else {
            Err(Error::BufferFull)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use rstest::rstest;

    #[rstest]
    #[case(123.456_f32,[0xca, 0x42, 0xf6, 0xe9, 0x79])]
    fn encode_float32<V: Encode, E: AsRef<[u8]> + Sized>(#[case] value: V, #[case] expected: E) {
        let expected = expected.as_ref();
        {
            let mut buf = vec![];
            let n = value.encode(&mut buf).unwrap();
            assert_eq!(buf, expected);
            assert_eq!(n, expected.len());
        }

        {
            let mut buf = vec![0xff; core::mem::size_of::<E>()];
            let n = value.encode_to_slice(buf.as_mut_slice()).unwrap();
            assert_eq!(&buf, expected);
            assert_eq!(n, expected.len());
        }
    }

    #[rstest]
    #[case(123456.789_f64,[0xcb, 0x40, 0xfe, 0x24, 0x0c, 0x9f, 0xbe, 0x76, 0xc9])]
    fn encode_float64<V: Encode, E: AsRef<[u8]> + Sized>(#[case] value: V, #[case] expected: E) {
        let expected = expected.as_ref();
        {
            let mut buf = vec![];
            let n = value.encode(&mut buf).unwrap();
            assert_eq!(buf, expected);
            assert_eq!(n, expected.len());
        }

        {
            let mut buf = vec![0xff; core::mem::size_of::<E>()];
            let n = value.encode_to_slice(buf.as_mut_slice()).unwrap();
            assert_eq!(&buf, expected);
            assert_eq!(n, expected.len());
        }
    }
}
