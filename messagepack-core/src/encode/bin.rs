use super::{Encode, Error, Result};
use crate::{formats::Format, io::IoWrite};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct BinaryEncoder<'blob>(pub &'blob [u8]);

impl<'blob> core::ops::Deref for BinaryEncoder<'blob> {
    type Target = &'blob [u8];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<W: IoWrite> Encode<W> for BinaryEncoder<'_> {
    fn encode(&self, writer: &mut W) -> Result<usize, <W as IoWrite>::Error> {
        let self_len = self.len();
        let format_len = match self_len {
            0x00..=0xff => {
                let cast = self_len as u8;
                writer.write_bytes(&[Format::Bin8.as_byte(), cast])?;
                Ok(2)
            }
            0x100..=0xffff => {
                let cast = (self_len as u16).to_be_bytes();
                writer.write_bytes(&[Format::Bin16.as_byte(), cast[0], cast[1]])?;
                Ok(3)
            }
            0x10000..=0xffffffff => {
                let cast = (self_len as u32).to_be_bytes();
                writer.write_bytes(&[
                    Format::Bin32.as_byte(),
                    cast[0],
                    cast[1],
                    cast[2],
                    cast[3],
                ])?;

                Ok(5)
            }
            _ => Err(Error::InvalidFormat),
        }?;

        writer.write_bytes(self.0)?;
        Ok(format_len + self_len)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(0xc4, 255_u8.to_be_bytes(),[0x12;255])]
    #[case(0xc5, 65535_u16.to_be_bytes(),[0x34;65535])]
    #[case(0xc6, 65536_u32.to_be_bytes(),[0x56;65536])]
    fn encode_str_sized<S: AsRef<[u8]>, D: AsRef<[u8]>>(
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

        let encoder = BinaryEncoder(data.as_ref());

        let mut buf = vec![];
        let n = encoder.encode(&mut buf).unwrap();

        assert_eq!(&buf, &expected);
        assert_eq!(n, expected.len());
    }
}
