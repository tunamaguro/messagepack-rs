use num_traits::ToPrimitive;

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

/// encode minimum byte size
#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub struct EncodeMinimizeFloat<N>(pub N);

impl<N> Encode for EncodeMinimizeFloat<N>
where
    N: ToPrimitive,
{
    fn encode<T>(&self, buf: &mut T) -> Result<usize>
    where
        T: Extend<u8>,
    {
        let n = &self.0;

        if let Some(v) = n.to_f32() {
            if v.is_normal() {
                return v.encode(buf);
            };
        };
        if let Some(v) = n.to_f64() {
            return v.encode(buf);
        };

        Err(Error::InvalidFormat)
    }
    fn encode_to_iter_mut<'a>(&self, buf: &mut impl Iterator<Item = &'a mut u8>) -> Result<usize> {
        let n = &self.0;

        if let Some(v) = n.to_f32() {
            if v.is_normal() {
                return v.encode_to_iter_mut(buf);
            };
        };
        if let Some(v) = n.to_f64() {
            return v.encode_to_iter_mut(buf);
        };

        Err(Error::InvalidFormat)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use rstest::rstest;

    #[rstest]
    #[case(123.456_f32,[Format::Float32.as_byte(), 0x42, 0xf6, 0xe9, 0x79])]
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
    #[case(123456.789_f64,[Format::Float64.as_byte(), 0x40, 0xfe, 0x24, 0x0c, 0x9f, 0xbe, 0x76, 0xc9])]
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

    #[rstest]
    // #[case(123.456_f64, [Format::Float32.as_byte(), 0x42, 0xf6, 0xe9, 0x79])]
    #[case(1e39_f64, [Format::Float64.as_byte(), 0x48,0x07,0x82,0x87,0xf4,0x9c,0x4a,0x1d])]
    fn encode_float_minimize<V: ToPrimitive, E: AsRef<[u8]> + Sized>(
        #[case] value: V,
        #[case] expected: E,
    ) {
        let expected = expected.as_ref();
        let encoder = EncodeMinimizeFloat(value);
        {
            let mut buf = vec![];
            let n = encoder.encode(&mut buf).unwrap();
            assert_eq!(buf, expected);
            assert_eq!(n, expected.len());
        }

        {
            let mut buf = vec![0xff; core::mem::size_of::<E>()];
            let n = encoder.encode_to_slice(buf.as_mut_slice()).unwrap();
            assert_eq!(&buf, expected);
            assert_eq!(n, expected.len());
        }
    }
}
