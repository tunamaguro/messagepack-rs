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
pub struct CompositeType {
    pub primitives: PrimitiveTypes,
    pub strings: StringTypes,
    pub arrays: ArrayTypes,
}
