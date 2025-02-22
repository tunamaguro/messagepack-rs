pub mod error;

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq)]
pub struct Serializer<'a, Buf>
where
    Buf: Iterator<Item = &'a mut u8>,
{
    buf: Buf,
}

impl<'a, Buf> Serializer<'a, Buf>
where
    Buf: Iterator<Item = &'a mut u8>,
{
    pub fn new(buf: Buf) -> Self {
        Self { buf }
    }
}

pub type Error = error::Error;
