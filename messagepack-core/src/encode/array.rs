use super::{Encode, Error, Result};
use crate::{formats::Format, io::IoWrite};

pub struct ArrayFormatEncoder(pub usize);

impl<W: IoWrite> Encode<W> for ArrayFormatEncoder {
    fn encode(&self, writer: &mut W) -> Result<usize, <W as IoWrite>::Error> {
        match self.0 {
            0x00..=0b1111 => {
                let cast = self.0 as u8;
                writer.write(&[Format::FixArray(cast).as_byte()])?;
                Ok(1)
            }
            0x10..=0xffff => {
                let cast = (self.0 as u16).to_be_bytes();
                writer.write(&[Format::Array16.as_byte(), cast[0], cast[1]])?;

                Ok(3)
            }
            0x10000..=0xffffffff => {
                let cast = (self.0 as u32).to_be_bytes();
                writer.write(&[
                    Format::Array32.as_byte(),
                    cast[0],
                    cast[1],
                    cast[2],
                    cast[3],
                ])?;

                Ok(5)
            }
            _ => Err(Error::InvalidFormat),
        }
    }
}

impl<W, V> Encode<W> for &[V]
where
    W: IoWrite,
    V: Encode<W>,
{
    fn encode(&self, writer: &mut W) -> Result<usize, <W as IoWrite>::Error> {
        let format_len = ArrayFormatEncoder(self.len()).encode(writer)?;
        let array_len = self
            .iter()
            .map(|v| v.encode(writer))
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
    fn encode_fix_array<V: Encode<Vec<u8>>, Array: AsRef<[V]>, E: AsRef<[u8]> + Sized>(
        #[case] value: Array,
        #[case] expected: E,
    ) {
        let expected = expected.as_ref();

        let mut buf = vec![];
        let n = value.as_ref().encode(&mut buf).unwrap();
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

        let mut buf = vec![];
        let n = data.as_ref().encode(&mut buf).unwrap();

        assert_eq!(&buf, &expected);
        assert_eq!(n, expected.len());
    }
}
