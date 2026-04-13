use std::collections::HashMap;

use messagepack_core::{Decode, Encode};
use rand::{
    RngExt,
    distr::{Distribution, StandardUniform},
};
use serde::{Deserialize, Serialize};

fn seed_rng() -> impl rand::Rng {
    use rand::SeedableRng;
    rand::rngs::SmallRng::seed_from_u64(31415)
}

pub trait BenchData {
    fn generate() -> Self;
    fn generate_vec(n: usize) -> Vec<Self>
    where
        Self: Sized,
    {
        core::iter::repeat_with(|| Self::generate())
            .take(n)
            .collect()
    }
}

#[derive(Debug, Serialize, Deserialize, Encode, Decode)]
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

impl BenchData for PrimitiveTypes {
    fn generate() -> Self {
        let mut rng = seed_rng();
        rng.random()
    }
    fn generate_vec(n: usize) -> Vec<Self>
    where
        Self: Sized,
    {
        let rng = seed_rng();
        rng.random_iter().take(n).collect()
    }
}

#[derive(Debug, Serialize, Deserialize, Encode, Decode)]
pub struct StrTypes {
    short: String,
    medium: String,
    long: String,
}

impl Distribution<StrTypes> for StandardUniform {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> StrTypes {
        let short_len = rng.random_range(0..256);
        let medium_len = rng.random_range(512..1024);
        let long_len = rng.random_range(1024..4096);
        StrTypes {
            short: rng.random_iter::<char>().take(short_len).collect(),
            medium: rng.random_iter::<char>().take(medium_len).collect(),
            long: rng.random_iter::<char>().take(long_len).collect(),
        }
    }
}

impl BenchData for StrTypes {
    fn generate() -> Self {
        let mut rng = seed_rng();
        rng.random()
    }

    fn generate_vec(n: usize) -> Vec<Self>
    where
        Self: Sized,
    {
        let rng = seed_rng();
        rng.random_iter().take(n).collect()
    }
}

#[derive(Debug, Serialize, Deserialize, Encode, Decode)]
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

impl BenchData for StrTypesBorrowed<'_> {
    fn generate() -> Self {
        Default::default()
    }
}

#[derive(Debug, Serialize, Deserialize, Encode, Decode)]
pub struct ArrayTypes {
    short: Vec<u8>,
    medium: Vec<u8>,
    long: Vec<u8>,
}

impl BenchData for ArrayTypes {
    fn generate() -> Self {
        let mut rng = seed_rng();
        rng.random()
    }

    fn generate_vec(n: usize) -> Vec<Self>
    where
        Self: Sized,
    {
        let rng = seed_rng();
        rng.random_iter().take(n).collect()
    }
}

impl Distribution<ArrayTypes> for StandardUniform {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> ArrayTypes {
        let short_len = rng.random_range(0..256);
        let medium_len = rng.random_range(512..1024);
        let long_len = rng.random_range(1024..4096);
        ArrayTypes {
            short: rng.random_iter().take(short_len).collect(),
            medium: rng.random_iter().take(medium_len).collect(),
            long: rng.random_iter().take(long_len).collect(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Encode, Decode)]
pub struct ByteType {
    #[serde(with = "serde_bytes")]
    #[msgpack(bytes)]
    short: Vec<u8>,
    #[serde(with = "serde_bytes")]
    #[msgpack(bytes)]
    medium: Vec<u8>,
    #[serde(with = "serde_bytes")]
    #[msgpack(bytes)]
    long: Vec<u8>,
}

impl Distribution<ByteType> for StandardUniform {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> ByteType {
        let short_len = rng.random_range(0..256);
        let medium_len = rng.random_range(512..1024);
        let long_len = rng.random_range(1024..4096);
        ByteType {
            short: rng.random_iter().take(short_len).collect(),
            medium: rng.random_iter().take(medium_len).collect(),
            long: rng.random_iter().take(long_len).collect(),
        }
    }
}

impl BenchData for ByteType {
    fn generate() -> Self {
        let mut rng = seed_rng();
        rng.random()
    }

    fn generate_vec(n: usize) -> Vec<Self>
    where
        Self: Sized,
    {
        let rng = seed_rng();
        rng.random_iter().take(n).collect()
    }
}

#[derive(Debug, Serialize, Deserialize, Encode, Decode)]
pub struct ByteTypeBorrowed<'a> {
    #[serde(with = "serde_bytes")]
    #[msgpack(bytes)]
    short: &'a [u8],
    #[serde(with = "serde_bytes")]
    #[msgpack(bytes)]
    medium: &'a [u8],
    #[serde(with = "serde_bytes")]
    #[msgpack(bytes)]
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

impl BenchData for ByteTypeBorrowed<'_> {
    fn generate() -> Self {
        Default::default()
    }
}

#[derive(Debug, Serialize, Deserialize, Encode, Decode)]
pub struct MapType {
    small: HashMap<i32, String>,
    medium: HashMap<i16, u64>,
    large: HashMap<i64, u16>,
}

impl Distribution<MapType> for StandardUniform {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> MapType {
        let small_len = rng.random_range(0..256);
        let medium_len = rng.random_range(512..1024);
        let large_len = rng.random_range(1024..4096);

        let mut small = HashMap::with_capacity(small_len);
        for _ in 0..small_len {
            let len = rng.random_range(0..256);
            let s: String = rng.random_iter::<char>().take(len).collect();
            small.insert(rng.random(), s);
        }

        let mut medium = HashMap::with_capacity(medium_len);
        for _ in 0..medium_len {
            medium.insert(rng.random(), rng.random());
        }

        let mut large = HashMap::with_capacity(large_len);
        for _ in 0..large_len {
            large.insert(rng.random(), rng.random());
        }

        MapType {
            small,
            medium,
            large,
        }
    }
}

impl BenchData for MapType {
    fn generate() -> Self {
        let mut rng = seed_rng();
        rng.random()
    }
    fn generate_vec(n: usize) -> Vec<Self>
    where
        Self: Sized,
    {
        let rng = seed_rng();
        rng.random_iter().take(n).collect()
    }
}

#[derive(Debug, Serialize, Deserialize, Encode, Decode)]
pub struct CompositeType {
    pub primitives: PrimitiveTypes,
    pub strings: StrTypes,
    pub bytes: ByteType,
    pub arrays: ArrayTypes,
    pub map: MapType,
}

impl BenchData for CompositeType {
    fn generate() -> Self {
        Self {
            primitives: PrimitiveTypes::generate(),
            strings: StrTypes::generate(),
            bytes: ByteType::generate(),
            arrays: ArrayTypes::generate(),
            map: MapType::generate(),
        }
    }
}
