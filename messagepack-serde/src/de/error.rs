use serde::de;

pub(crate) type CoreError = messagepack_core::decode::Error;

/// Error during deserialization
#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub enum Error {
    /// Core error
    Decode(CoreError),
    /// Recursion limit (nesting depth) exceeded
    RecursionLimitExceeded,
    #[cfg(not(feature = "std"))]
    /// Parse error
    Custom,
    #[cfg(feature = "std")]
    /// Parse error
    Message(String),
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::Decode(e) => e.fmt(f),
            Error::RecursionLimitExceeded => write!(f, "recursion limit exceeded"),
            #[cfg(not(feature = "std"))]
            Error::Custom => write!(f, "Cannot deserialize format"),
            #[cfg(feature = "std")]
            Error::Message(msg) => f.write_str(msg),
        }
    }
}

impl From<CoreError> for Error {
    fn from(err: CoreError) -> Self {
        Error::Decode(err)
    }
}

impl de::StdError for Error {}
impl de::Error for Error {
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
