use super::{Encode, Result};
use crate::{formats::Format, io::IoWrite};

impl<W: IoWrite> Encode<W> for bool {
    fn encode(&self, writer: &mut W) -> Result<usize, <W as IoWrite>::Error> {
        match self {
            true => {
                writer.write_bytes(&Format::True.as_slice())?;
                Ok(1)
            }
            false => {
                writer.write_bytes(&Format::False.as_slice())?;
                Ok(1)
            }
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
    fn encode_bool<V: Encode<Vec<u8>>, E: AsRef<[u8]> + Sized>(
        #[case] value: V,
        #[case] expected: E,
    ) {
        let expected = expected.as_ref();

        let mut buf = vec![];
        let n = value.encode(&mut buf).unwrap();
        assert_eq!(buf, expected);
        assert_eq!(n, expected.len());
    }
}
