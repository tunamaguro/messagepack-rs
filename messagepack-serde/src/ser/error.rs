use serde::ser;

pub(crate) type CoreError<T> = messagepack_core::encode::Error<T>;

/// Error during serialization
#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub enum Error<T> {
    /// Core error
    Encode(CoreError<T>),
    /// Try serialize  array or map but not passed length
    SeqLenNone,
    #[cfg(not(feature = "std"))]
    /// Parse error
    Custom,
    #[cfg(feature = "std")]
    /// Parse error
    Message(String),
}

impl<T: core::fmt::Display> core::fmt::Display for Error<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::Encode(e) => e.fmt(f),
            #[cfg(not(feature = "std"))]
            Error::Custom => write!(f, "Not match serializer format"),
            #[cfg(feature = "std")]
            Error::Message(msg) => f.write_str(msg),
            Error::SeqLenNone => write!(f, "array/map family must be provided length"),
        }
    }
}

impl<T> From<CoreError<T>> for Error<T> {
    fn from(err: CoreError<T>) -> Self {
        Error::Encode(err)
    }
}

impl<T> ser::StdError for Error<T> where T: core::error::Error {}
impl<E> ser::Error for Error<E>
where
    E: core::error::Error,
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
