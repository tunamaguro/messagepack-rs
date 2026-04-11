use serde::ser;

pub(crate) type CoreError<T> = messagepack_core::encode::Error<T>;

/// Error during serialization
#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub enum Error<T> {
    /// Core error
    Encode(CoreError<T>),
    /// Tried to serialize an array or map without a length while `alloc` is disabled.
    SeqLenNone,
    #[cfg(not(feature = "alloc"))]
    /// Custom serialization error.
    Custom,
    #[cfg(feature = "alloc")]
    /// Custom serialization error.
    Custom(alloc::string::String),
}

impl<T: core::fmt::Display> core::fmt::Display for Error<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::Encode(e) => e.fmt(f),
            #[cfg(not(feature = "alloc"))]
            Error::Custom => write!(f, "unknown error"),
            #[cfg(feature = "alloc")]
            Error::Custom(msg) => f.write_str(msg),
            Error::SeqLenNone => {
                write!(
                    f,
                    "array/map family must be provided length when `alloc` feature is disabled"
                )
            }
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
        #[cfg(not(feature = "alloc"))]
        {
            Self::Custom
        }

        #[cfg(feature = "alloc")]
        {
            use alloc::string::ToString;
            Self::Custom(msg.to_string())
        }
    }
}
