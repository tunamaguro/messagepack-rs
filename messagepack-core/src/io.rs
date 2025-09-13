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

#[cfg(not(any(test, feature = "std")))]
impl IoWrite for &mut [u8] {
    type Error = WError;

    fn write(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
        SliceWriter::from_slice(self).write(buf)
    }
}

#[cfg(all(not(test), feature = "alloc", not(feature = "std")))]
impl IoWrite for alloc::vec::Vec<u8> {
    type Error = core::convert::Infallible;

    fn write(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
        VecRefWriter::new(self).write(buf)
    }
}

#[cfg(feature = "alloc")]
mod vec_writer {
    use super::IoWrite;

    /// Simple writer that writes into a `&mut Vec<u8>`.
    pub struct VecRefWriter<'a> {
        vec: &'a mut alloc::vec::Vec<u8>,
    }

    impl<'a> VecRefWriter<'a> {
        /// Create a new writer
        pub fn new(vec: &'a mut alloc::vec::Vec<u8>) -> Self {
            Self { vec }
        }
    }

    impl IoWrite for VecRefWriter<'_> {
        type Error = core::convert::Infallible;

        fn write(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
            self.vec.extend_from_slice(buf);
            Ok(())
        }
    }

    /// Simple writer that writes into a `Vec<u8>`.
    pub struct VecWriter {
        vec: alloc::vec::Vec<u8>,
    }

    impl VecWriter {
        /// Create a new writer
        pub fn new() -> Self {
            Self {
                vec: alloc::vec::Vec::new(),
            }
        }
        /// Get the inner vector
        pub fn into_vec(self) -> alloc::vec::Vec<u8> {
            self.vec
        }
    }

    impl Default for VecWriter {
        fn default() -> Self {
            Self::new()
        }
    }

    impl IoWrite for VecWriter {
        type Error = core::convert::Infallible;
        fn write(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
            self.vec.extend_from_slice(buf);
            Ok(())
        }
    }
}
#[cfg(feature = "alloc")]
pub use vec_writer::{VecRefWriter, VecWriter};

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

/// Types used by decoder
pub enum Reference<'de, 'a> {
    /// Reference to a byte sequence that survives at least as long as the de
    Borrowed(&'de [u8]),
    /// Reference to a byte sequence that may be free soon
    Copied(&'a [u8]),
}

impl Reference<'_, '_> {
    /// Borrow the underlying bytes regardless of `Borrowed` or `Copied`.
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Reference::Borrowed(b) => b,
            Reference::Copied(b) => b,
        }
    }
}

/// decode input source
pub trait IoRead<'de> {
    /// Error type produced by the reader.
    type Error: core::error::Error + 'static;
    /// read exactly `len` bytes and consume
    fn read_slice<'a>(&'a mut self, len: usize) -> Result<Reference<'de, 'a>, Self::Error>;
}

/// Simple reader that reads from a byte slice.
pub struct SliceReader<'de> {
    /// current buffer
    cursor: &'de [u8],
}
impl<'de> SliceReader<'de> {
    /// create a new reader
    pub fn new(buf: &'de [u8]) -> Self {
        Self { cursor: buf }
    }

    /// Get the remaining, committed bytes (peeked bytes are not subtracted
    /// until `consume()` is called).
    pub fn rest(&self) -> &'de [u8] {
        self.cursor
    }
}

/// `SliceReader` Error
#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub enum RError {
    /// buffer is empty
    BufferEmpty,
}

impl core::fmt::Display for RError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            RError::BufferEmpty => write!(f, "Buffer is empty"),
        }
    }
}

impl core::error::Error for RError {}

impl<'de> IoRead<'de> for SliceReader<'de> {
    type Error = RError;

    #[inline]
    fn read_slice<'a>(&'a mut self, len: usize) -> Result<Reference<'de, 'a>, Self::Error> {
        let (read, rest) = self
            .cursor
            .split_at_checked(len)
            .ok_or(RError::BufferEmpty)?;
        self.cursor = rest;
        Ok(Reference::Borrowed(read))
    }
}

#[cfg(feature = "std")]
mod std_reader {
    use super::IoRead;

    /// Simple reader that reads from a `std::io::Read`.
    pub struct StdReader<R> {
        reader: R,
        buf: std::vec::Vec<u8>,
    }

    impl<R> StdReader<R>
    where
        R: std::io::Read,
    {
        /// create a new reader
        pub fn new(reader: R) -> Self {
            Self {
                reader,
                buf: std::vec::Vec::new(),
            }
        }
    }

    impl<'de, R> IoRead<'de> for StdReader<R>
    where
        R: std::io::Read,
    {
        type Error = std::io::Error;

        fn read_slice<'a>(
            &'a mut self,
            len: usize,
        ) -> Result<super::Reference<'de, 'a>, Self::Error> {
            if self.buf.len() < len {
                self.buf.resize(len, 0);
            };
            self.reader.read_exact(&mut self.buf[..len])?;

            Ok(super::Reference::Copied(&self.buf[..len]))
        }
    }
}
#[cfg(feature = "std")]
pub use std_reader::StdReader;

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
