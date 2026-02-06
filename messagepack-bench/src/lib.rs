use std::collections::HashMap;

use rand::{
    Rng,
    distr::{Alphanumeric, Distribution, StandardUniform},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PrimitiveTypes(usize, i8, i16, i32, i64, u8, u16, u32, u64, f32, f64);

impl Distribution<PrimitiveTypes> for StandardUniform {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> PrimitiveTypes {
        PrimitiveTypes(
            rng.random_range(usize::MIN..usize::MAX),
            rng.random(),
            rng.random(),
            rng.random(),
            rng.random(),
            rng.random(),
            rng.random(),
            rng.random(),
            rng.random(),
            rng.random(),
            rng.random(),
        )
    }
}

impl Default for PrimitiveTypes {
    fn default() -> Self {
        rand::random()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StrTypes {
    short: String,
    medium: String,
    long: String,
}

impl Default for StrTypes {
    fn default() -> Self {
        Self {
            short: include_str!("../data/lorem-ipsum.txt").into(),
            medium: include_str!("../data/jp-constitution.txt").into(),
            long: include_str!("../data/raven.txt").into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StrTypesBorrowed<'a> {
    short: &'a str,
    medium: &'a str,
    long: &'a str,
}

impl Default for StrTypesBorrowed<'_> {
    fn default() -> Self {
        Self {
            short: include_str!("../data/lorem-ipsum.txt"),
            medium: include_str!("../data/jp-constitution.txt"),
            long: include_str!("../data/raven.txt"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArrayTypes {
    short: Vec<u8>,
    medium: Vec<u8>,
    long: Vec<u8>,
}

impl Default for ArrayTypes {
    fn default() -> Self {
        Self {
            short: include_bytes!("../data/lorem-ipsum.txt").into(),
            medium: include_bytes!("../data/jp-constitution.txt").into(),
            long: include_bytes!("../data/raven.txt").into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ByteType {
    #[serde(with = "serde_bytes")]
    short: Vec<u8>,
    #[serde(with = "serde_bytes")]
    medium: Vec<u8>,
    #[serde(with = "serde_bytes")]
    long: Vec<u8>,
}

impl Default for ByteType {
    fn default() -> Self {
        Self {
            short: include_bytes!("../data/lorem-ipsum.txt").into(),
            medium: include_bytes!("../data/jp-constitution.txt").into(),
            long: include_bytes!("../data/raven.txt").into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ByteTypeBorrowed<'a> {
    #[serde(with = "serde_bytes")]
    short: &'a [u8],
    #[serde(with = "serde_bytes")]
    medium: &'a [u8],
    #[serde(with = "serde_bytes")]
    long: &'a [u8],
}

impl Default for ByteTypeBorrowed<'_> {
    fn default() -> Self {
        Self {
            short: include_bytes!("../data/lorem-ipsum.txt"),
            medium: include_bytes!("../data/jp-constitution.txt"),
            long: include_bytes!("../data/raven.txt"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MapType {
    small: HashMap<i32, String>,
    medium: HashMap<i16, u64>,
    large: HashMap<i64, u16>,
}

impl Distribution<MapType> for StandardUniform {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> MapType {
        const SMALL_LEN: usize = 32;
        const MEDIUM_LEN: usize = 128;
        const LARGE_LEN: usize = 512;

        let mut small = HashMap::with_capacity(SMALL_LEN);
        for _ in 0..SMALL_LEN {
            let len = rng.random_range(0..256);
            let s: String = rng
                .sample_iter(&Alphanumeric)
                .take(len)
                .map(char::from)
                .collect();
            small.insert(rng.random(), s);
        }

        let mut medium = HashMap::with_capacity(MEDIUM_LEN);
        for _ in 0..MEDIUM_LEN {
            medium.insert(rng.random(), rng.random());
        }

        let mut large = HashMap::with_capacity(LARGE_LEN);
        for _ in 0..LARGE_LEN {
            large.insert(rng.random(), rng.random());
        }

        MapType {
            small,
            medium,
            large,
        }
    }
}

impl Default for MapType {
    fn default() -> Self {
        rand::random()
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CompositeType {
    pub primitives: PrimitiveTypes,
    pub strings: StrTypes,
    pub bytes: ByteType,
    pub arrays: ArrayTypes,
    pub map: MapType,
}

#[cfg(test)]
mod tests {
    use super::*;
    use messagepack_serde::ser::to_slice;
    use rmp_serde::to_vec_named;

    #[test]
    fn str_size() {
        let s = StrTypes::default();

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
