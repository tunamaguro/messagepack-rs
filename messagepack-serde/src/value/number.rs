use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Visitor};

/// Represents any number, it could be int or float.
///
/// ## Example
///
/// ```rust
/// use serde::{Deserialize, Serialize};
/// use messagepack_serde::{from_slice,value::Number};
/// #[derive(Debug, Serialize, Deserialize, PartialEq)]
/// struct Data{
///     num: Number
/// }
/// let buf:&[u8] = &[0x81,0xa3,0x6e,0x75,0x6d,0x01]; // {"num":1}
/// let data = from_slice::<Data>(buf).unwrap();
/// assert_eq!(data.num,Number::PositiveInt(1));
///
/// let buf:&[u8] = &[0x81,0xa3,0x6e,0x75,0x6d,0xd0,0x85]; // {"num":-123}
/// let data = from_slice::<Data>(buf).unwrap();
/// assert_eq!(data.num,Number::NegativeInt(-123));
///
/// let buf:&[u8] = &[0x81,0xa3,0x6e,0x75,0x6d,0xcb,0x3f,0xf8,0x00,0x00,0x00,0x00,0x00,0x00]; // {"num":1.5}
/// let data = from_slice::<Data>(buf).unwrap();
/// assert_eq!(data.num,Number::Float(1.5));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Number {
    /// Always positive
    PositiveInt(u64),
    /// Always negative
    NegativeInt(i64),
    /// Represents `float 32` and `float 64`
    Float(f64),
}

impl Number {
    /// If the `Number` is unsigned int, returns `u64`.
    ///
    /// ```rust
    /// # use messagepack_serde::value::Number;
    ///
    /// let n = Number::from(1);
    /// assert_eq!(n.as_unsigned_int(),Some(1));
    ///
    /// let n = Number::try_from(1isize).unwrap();
    /// assert_eq!(n.as_unsigned_int(),Some(1));
    ///
    /// let n = Number::from(-1);
    /// assert_eq!(n.as_unsigned_int(),None);
    /// ```
    pub fn as_unsigned_int(&self) -> Option<u64> {
        match self {
            Number::PositiveInt(v) => Some(*v),
            Number::NegativeInt(v) => (*v).try_into().ok(),
            _ => None,
        }
    }

    /// If the `Number` is signed int, returns `i64`.
    ///
    /// ```rust
    /// # use messagepack_serde::value::Number;
    /// let n = Number::from(-1);
    /// assert_eq!(n.as_signed_int(),Some(-1));
    ///
    /// let n = Number::from(1);
    /// assert_eq!(n.as_signed_int(),Some(1));
    /// ```
    pub fn as_signed_int(&self) -> Option<i64> {
        match self {
            Number::PositiveInt(v) => i64::try_from(*v).ok(),
            Number::NegativeInt(v) => Some(*v),
            _ => None,
        }
    }

    /// If the `Number` is floating number, returns `f64`.
    ///
    /// ```rust
    /// # use messagepack_serde::value::Number;
    /// let n = Number::from(1.5);
    /// assert_eq!(n.as_float(),Some(1.5));
    ///
    /// let n = Number::from(1);
    /// assert_eq!(n.as_float(),None);
    /// ```
    pub fn as_float(&self) -> Option<f64> {
        match self {
            Number::Float(v) => Some(*v),
            _ => None,
        }
    }
}

impl From<u64> for Number {
    fn from(value: u64) -> Self {
        Number::PositiveInt(value)
    }
}

impl From<i64> for Number {
    fn from(value: i64) -> Self {
        u64::try_from(value)
            .map(Number::PositiveInt)
            .unwrap_or_else(|_| Number::NegativeInt(value))
    }
}

macro_rules! impl_from_num {
    ($from:ident, $cast:path) => {
        impl From<$from> for Number {
            fn from(value: $from) -> Self {
                Self::from(value as $cast)
            }
        }
    };
}

impl_from_num!(u8, u64);
impl_from_num!(u16, u64);
impl_from_num!(u32, u64);
impl_from_num!(i8, i64);
impl_from_num!(i16, i64);
impl_from_num!(i32, i64);

impl TryFrom<usize> for Number {
    type Error = core::num::TryFromIntError;
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        u64::try_from(value).map(Number::PositiveInt)
    }
}

impl TryFrom<isize> for Number {
    type Error = core::num::TryFromIntError;
    fn try_from(value: isize) -> Result<Self, Self::Error> {
        if let Ok(v) = i64::try_from(value) { return Ok(Number::from(v)) }

        u64::try_from(value).map(Self::from)
    }
}

impl From<f32> for Number {
    fn from(value: f32) -> Self {
        Self::from(value as f64)
    }
}

impl From<f64> for Number {
    fn from(value: f64) -> Self {
        Self::Float(value)
    }
}

impl Serialize for Number {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Number::PositiveInt(n) => serializer.serialize_u64(*n),
            Number::NegativeInt(n) => serializer.serialize_i64(*n),
            Number::Float(n) => serializer.serialize_f64(*n),
        }
    }
}

impl<'de> Deserialize<'de> for Number {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct NumberVisitor;
        impl Visitor<'_> for NumberVisitor {
            type Value = Number;
            fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                formatter.write_str("a number")
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Number::from(v))
            }

            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Number::from(v))
            }

            fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Number::from(v))
            }
        }

        deserializer.deserialize_any(NumberVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::from_slice;
    use rstest::rstest;

    #[rstest]
    #[case([0x05],5)]
    #[case([0xcd, 0xff, 0xff],u16::MAX.into())]
    #[case([0xce, 0xff, 0xff,0xff,0xff],u32::MAX.into())]
    #[case([0xcf, 0xff, 0xff,0xff,0xff,0xff, 0xff,0xff,0xff],u64::MAX)]
    fn decode_unsigned_int<Buf: AsRef<[u8]>>(#[case] input: Buf, #[case] expected: u64) {
        let num = from_slice::<Number>(input.as_ref()).unwrap();
        assert_eq!(num, Number::PositiveInt(expected));
    }

    #[rstest]
    #[case([0xe0],-32)]
    #[case([0xd0, 0x80],i8::MIN.into())]
    #[case([0xd1, 0x80, 0x00],i16::MIN.into())]
    #[case([0xd2, 0x80, 0x00, 0x00, 0x00],i32::MIN.into())]
    #[case([0xd3, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],i64::MIN)]
    fn decode_signed_int<Buf: AsRef<[u8]>>(#[case] input: Buf, #[case] expected: i64) {
        let num = from_slice::<Number>(input.as_ref()).unwrap();
        assert_eq!(num, Number::NegativeInt(expected));
    }

    #[rstest]
    #[case([0xca, 0x42, 0xf6, 0xe9, 0x79],123.456)]
    #[case([0xcb, 0x40, 0xfe, 0x24, 0x0c, 0x9f, 0xcb, 0x0c, 0x02],123456.789012)]
    fn decode_float<Buf: AsRef<[u8]>>(#[case] input: Buf, #[case] expected: f64) {
        let num = from_slice::<Number>(input.as_ref()).unwrap();
        match num {
            Number::Float(n) => {
                let diff = (n - expected).abs();
                assert!(diff < (1e-5))
            }
            _ => {
                panic!("Err")
            }
        }
    }
}
