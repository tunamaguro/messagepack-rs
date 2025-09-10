//! Minimal write abstraction used by encoders.

/// Minimal `Write`‑like trait used by encoders to avoid committing to a
/// specific I/O model.
pub trait IoWrite {
    /// Error type produced by the writer.
    type Error: core::error::Error;
    /// Write all bytes from `buf`.
    fn write(&mut self, buf: &[u8]) -> Result<(), Self::Error>;
}

/// `SliceWriter` Error
#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub enum WError {
    /// buffer is full
    BufferFull,
}

impl core::fmt::Display for WError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            WError::BufferFull => write!(f, "Buffer is full"),
        }
    }
}

impl core::error::Error for WError {}

/// Simple writer that writes into a mutable byte slice.
pub struct SliceWriter<'a> {
    buf: &'a mut [u8],
    cursor: usize,
}

impl<'a> SliceWriter<'a> {
    /// Create a new writer over the given buffer.
    pub fn from_slice(buf: &'a mut [u8]) -> Self {
        Self { buf, cursor: 0 }
    }

    fn len(&self) -> usize {
        self.buf.len() - self.cursor
    }
}

impl IoWrite for SliceWriter<'_> {
    type Error = WError;

    fn write(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
        if self.len() >= buf.len() {
            let to = &mut self.buf[self.cursor..self.cursor + buf.len()];
            to.copy_from_slice(buf);
            self.cursor += buf.len();
            Ok(())
        } else {
            Err(WError::BufferFull)
        }
    }
}

#[cfg(not(feature = "std"))]
impl<'a> IoWrite for &'a mut [u8] {
    type Error = WError;

    fn write(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
        SliceWriter::from_slice(self).write(buf)
    }
}

/// Simple writer that writes into a `Vec<u8>`.
#[cfg(feature = "alloc")]
pub struct VecWriter<'a> {
    vec: &'a mut alloc::vec::Vec<u8>,
}

#[cfg(feature = "alloc")]
impl<'a> VecWriter<'a> {
    /// Create a new writer
    pub fn new(vec: &'a mut alloc::vec::Vec<u8>) -> Self {
        Self { vec }
    }
}

#[cfg(feature = "alloc")]
impl IoWrite for VecWriter<'_> {
    type Error = core::convert::Infallible;

    fn write(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
        self.vec.extend_from_slice(buf);
        Ok(())
    }
}

#[cfg(all(feature = "alloc", not(feature = "std")))]
impl IoWrite for alloc::vec::Vec<u8> {
    type Error = core::convert::Infallible;

    fn write(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
        VecWriter::new(self).write(buf)
    }
}

#[cfg(any(test, feature = "std"))]
impl<W> IoWrite for W
where
    W: std::io::Write,
{
    type Error = std::io::Error;

    fn write(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
        self.write_all(buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn buffer_full() {
        let buf: &mut [u8] = &mut [0u8];
        let mut writer = SliceWriter::from_slice(buf);
        writer.write(&[1, 2]).unwrap();
    }
}
