use serde::ser;

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub enum Error {
    Encode(messagepack_core::encode::Error),
    #[cfg(not(feature = "std"))]
    Custom,
    #[cfg(feature = "std")]
    Message(String),
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::Encode(e) => e.fmt(f),
            #[cfg(not(feature = "std"))]
            Error::Custom => write!(f, "Not match serializer format"),
            #[cfg(feature = "std")]
            Error::Message(msg) => f.write_str(msg),
        }
    }
}

impl ser::StdError for Error {}
impl ser::Error for Error {
    #[allow(unused_variables)]
    fn custom<T>(msg: T) -> Self
    where
        T: core::fmt::Display,
    {
        #[cfg(not(feature = "std"))]
        {
            Self::Custom
        }

        #[cfg(feature = "std")]
        {
            Self::Message(msg.to_string())
        }
    }
}
