use rand::distr::{Distribution, StandardUniform};
use serde::Serialize;

#[derive(Debug, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct PrimitiveTypes {
    usize: usize,
    i8: i8,
    i16: i16,
    i32: i32,
    i64: i64,
    u8: u8,
    u16: u16,
    u32: u32,
    u64: u64,
}

impl Distribution<PrimitiveTypes> for StandardUniform {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> PrimitiveTypes {
        PrimitiveTypes {
            usize: rng.random_range(usize::MIN..usize::MAX),
            i8: rng.random(),
            i16: rng.random(),
            i32: rng.random(),
            i64: rng.random(),
            u8: rng.random(),
            u16: rng.random(),
            u32: rng.random(),
            u64: rng.random(),
        }
    }
}

#[derive(Debug, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct StringTypes {
    short: &'static str,
    medium: &'static str,
    long: &'static str,
}

impl Default for StringTypes {
    fn default() -> Self {
        Self {
            short: include_str!("../data/lorem-ipsum.txt"),
            medium: include_str!("../data/jp-constitution.txt"),
            long: include_str!("../data/raven.txt"),
        }
    }
}

#[derive(Debug, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct ArrayTypes {
    short: &'static [u8],
    medium: &'static [u8],
    long: &'static [u8],
}

impl Default for ArrayTypes {
    fn default() -> Self {
        Self {
            short: include_bytes!("../data/lorem-ipsum.txt"),
            medium: include_bytes!("../data/jp-constitution.txt"),
            long: include_bytes!("../data/raven.txt"),
        }
    }
}

#[derive(Debug, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct ByteType {
    #[serde(with = "serde_bytes")]
    short: &'static [u8],
    #[serde(with = "serde_bytes")]
    medium: &'static [u8],
    #[serde(with = "serde_bytes")]
    long: &'static [u8],
}

impl Default for ByteType {
    fn default() -> Self {
        Self {
            short: include_bytes!("../data/lorem-ipsum.txt"),
            medium: include_bytes!("../data/jp-constitution.txt"),
            long: include_bytes!("../data/raven.txt"),
        }
    }
}

#[derive(Debug, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct CompositeType {
    pub primitives: PrimitiveTypes,
    pub strings: StringTypes,
    pub arrays: ArrayTypes,
}

impl Default for CompositeType {
    fn default() -> Self {
        let p = rand::random::<PrimitiveTypes>();
        Self {
            primitives: p,
            arrays: Default::default(),
            strings: Default::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use messagepack_serde::ser::to_slice;
    use rmp_serde::to_vec_named;

    #[test]
    fn str_size() {
        let s = StringTypes::default();

        let rmp = to_vec_named(&s).unwrap();

        let buf = &mut [0_u8; 4096 * 10];
        let len = to_slice(&s, buf).unwrap();
        assert_eq!(rmp.len(), len);
    }

    #[test]
    fn byte_size() {
        let s = ArrayTypes::default();

        let rmp = to_vec_named(&s).unwrap();

        let buf = &mut [0_u8; 4096 * 10];
        let len = to_slice(&s, buf).unwrap();
        assert_eq!(rmp.len(), len);
    }
}
