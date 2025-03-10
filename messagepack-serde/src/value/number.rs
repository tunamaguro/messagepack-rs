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
/// assert_eq!(data.num,Number::UnsignedInt(1));
///
/// let buf:&[u8] = &[0x81,0xa3,0x6e,0x75,0x6d,0xd0,0x85]; // {"num":-123}
/// let data = from_slice::<Data>(buf).unwrap();
/// assert_eq!(data.num,Number::SignedInt(-123));
///
/// let buf:&[u8] = &[0x81,0xa3,0x6e,0x75,0x6d,0xcb,0x3f,0xf8,0x00,0x00,0x00,0x00,0x00,0x00]; // {"num":1.5}
/// let data = from_slice::<Data>(buf).unwrap();
/// assert_eq!(data.num,Number::Float(1.5));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Number {
    UnsignedInt(u64),
    SignedInt(i64),
    Float(f64),
}

impl Number {
    pub fn as_unsigned_int(&self) -> Option<u64> {
        match self {
            Number::UnsignedInt(v) => Some(*v),
            _ => None,
        }
    }
    pub fn as_signed_int(&self) -> Option<i64> {
        match self {
            Number::SignedInt(v) => Some(*v),
            _ => None,
        }
    }
    pub fn as_float(&self) -> Option<f64> {
        match self {
            Number::Float(v) => Some(*v),
            _ => None,
        }
    }
}

macro_rules! impl_from_num {
    ($from:ident,$ty:path,$cast:path) => {
        impl From<$from> for Number {
            fn from(value: $from) -> Self {
                $ty(value as $cast)
            }
        }
    };
}

impl_from_num!(u8, Number::UnsignedInt, u64);
impl_from_num!(u16, Number::UnsignedInt, u64);
impl_from_num!(u32, Number::UnsignedInt, u64);
impl_from_num!(u64, Number::UnsignedInt, u64);
impl_from_num!(i8, Number::SignedInt, i64);
impl_from_num!(i16, Number::SignedInt, i64);
impl_from_num!(i32, Number::SignedInt, i64);
impl_from_num!(i64, Number::SignedInt, i64);
impl_from_num!(f32, Number::Float, f64);
impl_from_num!(f64, Number::Float, f64);

impl Serialize for Number {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Number::UnsignedInt(n) => serializer.serialize_u64(*n),
            Number::SignedInt(n) => serializer.serialize_i64(*n),
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
                Ok(Number::UnsignedInt(v))
            }

            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Number::SignedInt(v))
            }

            fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Number::Float(v))
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
        assert_eq!(num, Number::UnsignedInt(expected));
    }

    #[rstest]
    #[case([0xe0],-32)]
    #[case([0xd0, 0x7f],i8::MAX.into())]
    #[case([0xd1, 0x7f, 0xff],i16::MAX.into())]
    #[case([0xd2, 0x80, 0x00, 0x00, 0x00],i32::MIN.into())]
    #[case([0xd3, 0x7f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff],i64::MAX)]
    fn decode_signed_int<Buf: AsRef<[u8]>>(#[case] input: Buf, #[case] expected: i64) {
        let num = from_slice::<Number>(input.as_ref()).unwrap();
        assert_eq!(num, Number::SignedInt(expected));
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
