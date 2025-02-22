use super::{Encode, Error, Result};
use crate::formats::Format;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ArrayEncoder<'array, V>(&'array [V]);

impl<'array, V> core::ops::Deref for ArrayEncoder<'array, V> {
    type Target = &'array [V];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<V: Encode> Encode for ArrayEncoder<'_, V>
where
    V: Encode,
{
    fn encode<T>(&self, buf: &mut T) -> Result<usize>
    where
        T: Extend<u8>,
    {
        let self_len = self.len();
        let format_len = match self_len {
            0x00..=0b1111 => {
                let cast = self_len as u8;
                let it = Format::FixArray(cast);
                buf.extend(it);
                Ok(1)
            }
            0x10..=0xffff => {
                let cast = self_len as u16;
                let it = Format::Array16.into_iter().chain(cast.to_be_bytes());
                buf.extend(it);
                Ok(3)
            }
            0x10000..=0xffffffff => {
                let cast = self_len as u32;
                let it = Format::Array32.into_iter().chain(cast.to_be_bytes());
                buf.extend(it);
                Ok(5)
            }
            _ => Err(Error::InvalidFormat),
        }?;

        let array_len = self
            .iter()
            .map(|v| v.encode(buf))
            .try_fold(0, |acc, v| v.map(|n| acc + n))?;
        Ok(format_len + array_len)
    }
    fn encode_to_iter_mut<'a>(&self, buf: &mut impl Iterator<Item = &'a mut u8>) -> Result<usize> {
        let self_len = self.len();
        let format_len = match self_len {
            0x00..=0b1111 => {
                const SIZE: usize = 1;
                let cast = self_len as u8;
                let it = &mut Format::FixArray(cast).into_iter();
                for (byte, to) in it.zip(buf.by_ref()) {
                    *to = byte;
                }

                if it.next().is_none() {
                    Ok(SIZE)
                } else {
                    Err(Error::BufferFull)
                }
            }
            0x10..=0xffff => {
                const SIZE: usize = 3;
                let cast = self_len as u16;
                let it = &mut Format::Array16.into_iter().chain(cast.to_be_bytes());
                for (byte, to) in it.zip(buf.by_ref()) {
                    *to = byte;
                }

                if it.next().is_none() {
                    Ok(SIZE)
                } else {
                    Err(Error::BufferFull)
                }
            }
            0x10000..=0xffffffff => {
                const SIZE: usize = 5;
                let cast = self_len as u32;
                let it = &mut Format::Array32.into_iter().chain(cast.to_be_bytes());
                for (byte, to) in it.zip(buf.by_ref()) {
                    *to = byte;
                }

                if it.next().is_none() {
                    Ok(SIZE)
                } else {
                    Err(Error::BufferFull)
                }
            }
            _ => Err(Error::InvalidFormat),
        }?;
        let array_len = self
            .iter()
            .map(|v| v.encode_to_iter_mut(buf))
            .try_fold(0, |acc, v| v.map(|n| acc + n))?;
        Ok(format_len + array_len)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case([1u8, 2u8, 3u8],[0x93, 0x01, 0x02, 0x03])]
    fn encode_fix_array<V: Encode, Array: AsRef<[V]>, E: AsRef<[u8]> + Sized>(
        #[case] value: Array,
        #[case] expected: E,
    ) {
        let expected = expected.as_ref();
        let encoder = ArrayEncoder(value.as_ref());
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
        {
            let mut buf = vec![];
            let n = encoder.encode(&mut buf).unwrap();

            assert_eq!(&buf, &expected);
            assert_eq!(n, expected.len());
        }

        {
            let mut buf = vec![0xff; expected.len()];
            let n = encoder.encode_to_slice(buf.as_mut_slice()).unwrap();
            assert_eq!(&buf, &expected);
            assert_eq!(n, expected.len());
        }
    }
}
