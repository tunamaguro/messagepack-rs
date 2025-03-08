pub trait IoWrite {
    type Error: core::error::Error;
    fn write_byte(&mut self, byte: u8) -> Result<(), Self::Error>;
    fn write_bytes(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
        for byte in buf {
            self.write_byte(*byte)?;
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

pub struct SliceWriter<'a> {
    buf: &'a mut [u8],
    cursor: usize,
}

impl<'a> SliceWriter<'a> {
    pub fn from_slice(buf: &'a mut [u8]) -> Self {
        Self { buf, cursor: 0 }
    }

    fn len(&self) -> usize {
        self.buf.len() - self.cursor
    }
}

impl IoWrite for SliceWriter<'_> {
    type Error = WError;

    fn write_byte(&mut self, byte: u8) -> Result<(), Self::Error> {
        if self.len() >= 1 {
            self.buf[self.cursor] = byte;
            self.cursor += 1;
            Ok(())
        } else {
            Err(WError::BufferFull)
        }
    }

    fn write_bytes(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
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

#[cfg(any(test, feature = "std"))]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn buffer_full() {
        let buf: &mut [u8] = &mut [0u8];
        let mut writer = SliceWriter::from_slice(buf);
        writer.write_bytes(&[1, 2]).unwrap();
    }
}
