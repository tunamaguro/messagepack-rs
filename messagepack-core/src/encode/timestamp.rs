use super::{Encode, Result, extension::ExtensionEncoder};
use crate::{
    io::IoWrite,
    timestamp::{TIMESTAMP_EXTENSION_TYPE, Timestamp32, Timestamp64, Timestamp96},
};

impl<W: IoWrite> Encode<W> for Timestamp32 {
    fn encode(&self, writer: &mut W) -> Result<usize, W::Error> {
        let buf = self.to_buf();
        ExtensionEncoder::new(TIMESTAMP_EXTENSION_TYPE, &buf).encode(writer)
    }
}

impl<W: IoWrite> Encode<W> for Timestamp64 {
    fn encode(&self, writer: &mut W) -> Result<usize, W::Error> {
        let buf = self.to_buf();
        ExtensionEncoder::new(TIMESTAMP_EXTENSION_TYPE, &buf).encode(writer)
    }
}

impl<W: IoWrite> Encode<W> for Timestamp96 {
    fn encode(&self, writer: &mut W) -> Result<usize, W::Error> {
        let buf = self.to_buf();
        ExtensionEncoder::new(TIMESTAMP_EXTENSION_TYPE, &buf).encode(writer)
    }
}
