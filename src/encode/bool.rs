use super::{Encode, Error, Result};
use crate::formats::Format;

impl Encode for bool {
    fn encode<T>(&self, buf: &mut T) -> Result<usize>
    where
        T: Extend<u8>,
    {
        match self {
            true => {
                buf.extend(Format::True);
                Ok(1)
            }
            false => {
                buf.extend(Format::False);
                Ok(1)
            }
        }
    }
    fn encode_to_iter_mut<'a>(&self, buf: &mut impl Iterator<Item = &'a mut u8>) -> Result<usize> {
        if let Some(v) = buf.next() {
            match self {
                true => *v = Format::True.as_byte(),
                false => *v = Format::False.as_byte(),
            };
            Ok(1)
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
    #[case(true,[0xc3])]
    #[case(false,[0xc2])]
    fn encode_bool<V: Encode, E: AsRef<[u8]> + Sized>(#[case] value: V, #[case] expected: E) {
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
