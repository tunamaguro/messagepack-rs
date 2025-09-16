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

#[cfg(all(not(test), not(feature = "std")))]
impl IoWrite for &mut [u8] {
    type Error = WError;

    fn write(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
        let this = core::mem::replace(self, &mut []);

        let (written, rest) = this
            .split_at_mut_checked(buf.len())
            .ok_or(WError::BufferFull)?;
        written.copy_from_slice(buf);
        *self = rest;

        Ok(())
    }
}

#[cfg(all(not(test), feature = "alloc", not(feature = "std")))]
mod alloc_without_std {
    use super::{IoWrite, vec_writer::VecRefWriter};
    impl IoWrite for alloc::vec::Vec<u8> {
        type Error = core::convert::Infallible;

        fn write(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
            VecRefWriter::new(self).write(buf)
        }
    }

    impl IoWrite for &mut alloc::vec::Vec<u8> {
        type Error = core::convert::Infallible;

        fn write(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
            VecRefWriter::new(self).write(buf)
        }
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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

#[cfg(feature = "alloc")]
mod iter_reader {
    use crate::io::RError;

    use super::IoRead;

    /// Reader that reads from a iterator
    pub struct IterReader<I> {
        it: I,
        buf: alloc::vec::Vec<u8>,
    }

    impl<I> IterReader<I>
    where
        I: Iterator<Item = u8>,
    {
        /// create new reader
        pub fn new(it: I) -> Self {
            Self {
                it: it.into_iter(),
                buf: alloc::vec::Vec::new(),
            }
        }
    }
    impl<'de, I> IoRead<'de> for IterReader<I>
    where
        I: Iterator<Item = u8>,
    {
        type Error = RError;
        fn read_slice<'a>(
            &'a mut self,
            len: usize,
        ) -> Result<super::Reference<'de, 'a>, Self::Error> {
            self.buf.clear();
            if self.buf.capacity() < len {
                self.buf.reserve(len - self.buf.capacity());
            }

            self.buf.extend(self.it.by_ref().take(len));
            if self.buf.len() != len {
                return Err(RError::BufferEmpty);
            };

            Ok(super::Reference::Copied(&self.buf[..len]))
        }
    }
}
#[cfg(feature = "alloc")]
pub use iter_reader::IterReader;

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

    #[test]
    fn slice_reader_reads_and_advances() {
        // Arrange: make a reader over a fixed slice
        let input: &[u8] = &[1, 2, 3, 4, 5];
        let mut reader = SliceReader::new(input);

        // Act: read exact 2 bytes, then 3 bytes
        {
            // Keep the first borrow in a narrower scope
            let a = reader.read_slice(2).expect("read 2 bytes");
            assert_eq!(a.as_bytes(), &[1, 2]);
        }
        let b = reader.read_slice(3).expect("read 3 bytes");
        // Assert: returned slices match and rest is empty
        assert_eq!(b.as_bytes(), &[3, 4, 5]);
        assert_eq!(reader.rest(), &[]);
    }

    #[test]
    fn slice_reader_returns_error_on_overshoot() {
        // Arrange
        let input: &[u8] = &[10, 20];
        let mut reader = SliceReader::new(input);

        // Act: first read consumes all bytes
        let first = reader.read_slice(2).expect("read 2 bytes");
        assert_eq!(first.as_bytes(), &[10, 20]);

        // Assert: second read fails with BufferEmpty
        assert!(matches!(reader.read_slice(1), Err(RError::BufferEmpty)));
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn iter_reader_reads_exact_length() {
        // Arrange: iterator with 4 items
        let it = [7u8, 8, 9, 10].into_iter();
        let mut reader = IterReader::new(it);

        // Act: read 3 then 1
        {
            let part1 = reader.read_slice(3).expect("read 3 bytes");
            assert_eq!(part1.as_bytes(), &[7, 8, 9]);
        }
        let part2 = reader.read_slice(1).expect("read 1 byte");

        // Assert
        assert_eq!(part2.as_bytes(), &[10]);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn iter_reader_returns_error_when_insufficient() {
        // Arrange: iterator shorter than requested length
        let it = [1u8, 2].into_iter();
        let mut reader = IterReader::new(it);

        // Act + Assert: request more than available -> error
        assert!(matches!(reader.read_slice(3), Err(RError::BufferEmpty)));
    }
}
