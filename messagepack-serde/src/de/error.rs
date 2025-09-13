use serde::de;

pub(crate) type CoreError<E> = messagepack_core::decode::Error<E>;

/// Error during deserialization
#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub enum Error<E> {
    /// Core error
    Decode(CoreError<E>),
    /// Recursion limit (nesting depth) exceeded
    RecursionLimitExceeded,
    #[cfg(not(feature = "std"))]
    /// Parse error
    Custom,
    #[cfg(feature = "std")]
    /// Parse error
    Message(String),
}

impl<E> core::fmt::Display for Error<E>
where
    E: core::fmt::Display,
{
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

impl<E> From<CoreError<E>> for Error<E> {
    fn from(err: CoreError<E>) -> Self {
        Error::Decode(err)
    }
}

impl<E> de::StdError for Error<E>
where
    E: core::error::Error + 'static,
{
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Error::Decode(e) => Some(e),
            _ => None,
        }
    }
}
impl<E> de::Error for Error<E>
where
    E: core::error::Error + 'static,
{
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
