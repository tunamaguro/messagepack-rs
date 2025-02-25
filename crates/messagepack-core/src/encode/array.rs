use core::{cell::RefCell, marker::PhantomData};

use super::{Encode, Error, Result};
use crate::{formats::Format, io::IoWrite};

pub struct ArrayFormatEncoder(pub usize);
impl ArrayFormatEncoder {
    pub fn new(size: usize) -> Self {
        Self(size)
    }
}
impl<W: IoWrite> Encode<W> for ArrayFormatEncoder {
    fn encode(&self, writer: &mut W) -> Result<usize, <W as IoWrite>::Error> {
        match self.0 {
            0x00..=0b1111 => {
                let cast = self.0 as u8;
                let it = Format::FixArray(cast);
                writer.write_iter(it)?;
                Ok(1)
            }
            0x10..=0xffff => {
                let cast = self.0 as u16;
                let it = Format::Array16.into_iter().chain(cast.to_be_bytes());
                writer.write_iter(it)?;
                Ok(3)
            }
            0x10000..=0xffffffff => {
                let cast = self.0 as u32;
                let it = Format::Array32.into_iter().chain(cast.to_be_bytes());
                writer.write_iter(it)?;
                Ok(5)
            }
            _ => Err(Error::InvalidFormat),
        }
    }
}

pub struct ArrayDataEncoder<I, V> {
    data: RefCell<I>,
    _phantom: PhantomData<(I, V)>,
}

impl<I, V> ArrayDataEncoder<I, V> {
    pub fn new(data: I) -> Self {
        ArrayDataEncoder {
            data: RefCell::new(data),
            _phantom: Default::default(),
        }
    }
}

impl<W, I, V> Encode<W> for ArrayDataEncoder<I, V>
where
    W: IoWrite,
    I: Iterator<Item = V>,
    V: Encode<W>,
{
    fn encode(&self, writer: &mut W) -> Result<usize, <W as IoWrite>::Error> {
        let array_len = self
            .data
            .borrow_mut()
            .by_ref()
            .map(|v| v.encode(writer))
            .try_fold(0, |acc, v| v.map(|n| acc + n))?;
        Ok(array_len)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ArrayEncoder<'array, V>(&'array [V]);

impl<'array, V> core::ops::Deref for ArrayEncoder<'array, V> {
    type Target = &'array [V];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<W, V> Encode<W> for ArrayEncoder<'_, V>
where
    W: IoWrite,
    V: Encode<W>,
{
    fn encode(&self, writer: &mut W) -> Result<usize, <W as IoWrite>::Error> {
        let self_len = self.len();
        let format_len = ArrayFormatEncoder(self_len).encode(writer)?;

        let array_len = ArrayDataEncoder::new(self.iter()).encode(writer)?;
        Ok(format_len + array_len)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case([1u8, 2u8, 3u8],[0x93, 0x01, 0x02, 0x03])]
    fn encode_fix_array<V: Encode<Vec<u8>>, Array: AsRef<[V]>, E: AsRef<[u8]> + Sized>(
        #[case] value: Array,
        #[case] expected: E,
    ) {
        let expected = expected.as_ref();
        let encoder = ArrayEncoder(value.as_ref());

        let mut buf = vec![];
        let n = encoder.encode(&mut buf).unwrap();
        assert_eq!(buf, expected);
        assert_eq!(n, expected.len());
    }

    #[rstest]
    #[case(0xdc, 65535_u16.to_be_bytes(),[0x34;65535])]
    #[case(0xdd, 65536_u32.to_be_bytes(),[0x56;65536])]
    fn encode_array_sized<S: AsRef<[u8]>, D: AsRef<[u8]>>(
        #[case] marker: u8,
        #[case] size: S,
        #[case] data: D,
    ) {
        let expected = marker
            .to_be_bytes()
            .iter()
            .chain(size.as_ref())
            .chain(data.as_ref())
            .cloned()
            .collect::<Vec<u8>>();

        let encoder = ArrayEncoder(data.as_ref());

        let mut buf = vec![];
        let n = encoder.encode(&mut buf).unwrap();

        assert_eq!(&buf, &expected);
        assert_eq!(n, expected.len());
    }
}
