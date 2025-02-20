mod bool;

/// Messagepack Encode Error
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Error {
    /// buffer is full
    BufferFull,
}

type Result<T> = ::core::result::Result<T, Error>;

/// A type which can be encoded to MessagePack
pub trait Encode {
    /// encode to MessagePack
    fn encode(&self) -> Result<impl Iterator<Item = u8>>;

    /// encode to slice
    fn encode_to_slice(&self, buf: &mut [u8]) -> Result<()> {
        let encoded = self.encode()?;
        for (idx, byte) in encoded.enumerate() {
            if let Some(v) = buf.get_mut(idx) {
                *v = byte;
            } else {
                return Err(Error::BufferFull);
            }
        }

        Ok(())
    }
}
