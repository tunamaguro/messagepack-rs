use core::marker::PhantomData;

pub trait IoWrite {
    type Error: core::error::Error;
    fn write_byte(&mut self, byte: u8) -> Result<(), Self::Error>;
    fn write_iter<I: IntoIterator<Item = u8>>(&mut self, iter: I) -> Result<(), Self::Error> {
        for byte in iter {
            self.write_byte(byte)?;
        }

        Ok(())
    }
}

pub trait IoRead {
    type Error: core::error::Error;
    fn read_byte(&mut self) -> Result<u8, Self::Error>;
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        for to in buf {
            *to = self.read_byte()?;
        }
        Ok(())
    }
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

pub struct SliceWriter<'a, I> {
    buf: I,
    _phantom: PhantomData<&'a ()>,
}

impl<'a, I> SliceWriter<'a, I>
where
    I: Iterator<Item = &'a mut u8>,
{
    pub fn new(buf: I) -> Self {
        Self {
            buf,
            _phantom: Default::default(),
        }
    }
}

impl<'a> SliceWriter<'a, core::slice::IterMut<'a, u8>> {
    pub fn from_slice(buf: &'a mut [u8]) -> Self {
        Self::new(buf.iter_mut())
    }
}

impl<'a, I> IoWrite for SliceWriter<'a, I>
where
    I: Iterator<Item = &'a mut u8>,
{
    type Error = WError;

    fn write_byte(&mut self, byte: u8) -> Result<(), Self::Error> {
        match self.buf.next() {
            Some(to) => {
                *to = byte;
                Ok(())
            }
            None => Err(WError::BufferFull),
        }
    }
}

/// `SliceReader` Error
#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub enum RError {
    /// end of data
    EofData,
}

impl core::fmt::Display for RError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            RError::EofData => write!(f, "Eof data"),
        }
    }
}

impl core::error::Error for RError {}

pub struct SliceReader<'a, I> {
    buf: I,
    _phantom: PhantomData<&'a ()>,
}

impl<'a, I> SliceReader<'a, I>
where
    I: Iterator<Item = &'a u8>,
{
    pub fn new(buf: I) -> Self {
        Self {
            buf,
            _phantom: Default::default(),
        }
    }
}

impl<'a> SliceReader<'a, core::slice::Iter<'a, u8>> {
    pub fn from_slice(buf: &'a [u8]) -> Self {
        Self {
            buf: buf.iter(),
            _phantom: Default::default(),
        }
    }
}

impl<'a, I> IoRead for SliceReader<'a, I>
where
    I: Iterator<Item = &'a u8>,
{
    type Error = RError;
    fn read_byte(&mut self) -> Result<u8, Self::Error> {
        match self.buf.next() {
            Some(byte) => Ok(*byte),
            None => Err(RError::EofData),
        }
    }
}

#[cfg(feature = "std")]
impl<W> IoWrite for W
where
    W: std::io::Write,
{
    type Error = std::io::Error;

    fn write_byte(&mut self, byte: u8) -> Result<(), Self::Error> {
        match self.write_all(&[byte]) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }
}

#[cfg(feature = "std")]
impl<R> IoRead for R
where
    R: std::io::Read,
{
    type Error = std::io::Error;

    fn read_byte(&mut self) -> Result<u8, Self::Error> {
        let mut buf = [0u8; 1];
        self.read_exact(&mut buf)?;

        Ok(buf[0])
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        self.read_exact(buf)
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
        writer.write_iter([1, 2]).unwrap();
    }

    #[test]
    #[should_panic]
    fn eof_of_data() {
        let buf: &mut [u8] = &mut [0u8];
        let mut reader = SliceReader::from_slice(buf);
        reader.read_exact(&mut [255; 2]).unwrap();
    }
}
