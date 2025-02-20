use super::{Encode, Result};
use crate::formats;

impl Encode for bool {
    fn encode(&self) -> Result<impl Iterator<Item = u8>> {
        let iter = match self {
            true => [formats::TRUE],
            false => [formats::FALSE],
        };

        Ok(iter.into_iter())
    }
}
