//! Boolean encoders.

use super::{Encode, Result};
use crate::{formats::Format, io::IoWrite};

impl Encode for bool {
    fn encode<W: IoWrite>(&self, writer: &mut W) -> Result<usize, <W as IoWrite>::Error> {
        match self {
            true => {
                writer.write(&Format::True.as_slice())?;
                Ok(1)
            }
            false => {
                writer.write(&Format::False.as_slice())?;
                Ok(1)
            }
        }
    }
}

impl Encode for core::sync::atomic::AtomicBool {
    fn encode<W: IoWrite>(&self, writer: &mut W) -> Result<usize, W::Error> {
        self.load(core::sync::atomic::Ordering::Relaxed)
            .encode(writer)
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

        let mut buf = vec![];
        let n = value.encode(&mut buf).unwrap();
        assert_eq!(buf, expected);
        assert_eq!(n, expected.len());
    }
}
