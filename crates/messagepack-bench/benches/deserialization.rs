#![allow(unexpected_cfgs)]

#[cfg(not(codspeed))]
use divan::counter::BytesCount;
use messagepack_bench::{
    ArrayTypes, ByteType, CompositeType, MapType, PrimitiveTypes, StrTypes, StrTypesBorrowed,
};
use serde::{Serialize, de::DeserializeOwned};
use std::iter::repeat_with;

#[global_allocator]
static ALLOC: divan::AllocProfiler = divan::AllocProfiler::system();

fn main() {
    // Run registered benchmarks.
    divan::main();
}

const LENS: &[usize] = &[1024];
const BUFFER_SIZE: usize = (2u32.pow(16)) as usize;

#[divan::bench(
    types = [ArrayTypes, ByteType, CompositeType, MapType, PrimitiveTypes, StrTypes],
    consts = LENS
)]
fn deserialize_messagepack_serde<
    T: Serialize + DeserializeOwned + Default + Sync,
    const N: usize,
>(
    #[allow(unused_mut)] mut bencher: divan::Bencher,
) {
    let s = repeat_with(|| T::default()).take(N).collect::<Vec<_>>();
    let mut buf = vec![0u8; BUFFER_SIZE * N];
    let buf_len = messagepack_serde::to_slice(&s, &mut buf).unwrap();

    #[cfg(not(codspeed))]
    {
        bencher = bencher.counter(BytesCount::of_slice(&buf))
    }

    bencher.bench_local(|| {
        let buf = core::hint::black_box(&buf[..buf_len]);
        messagepack_serde::from_slice::<Vec<T>>(buf).unwrap()
    });
}

#[divan::bench]
fn deserialize_borrowed_messagepack_serde(#[allow(unused_mut)] mut bencher: divan::Bencher) {
    let s = StrTypesBorrowed::default();
    let mut buf = vec![0u8; BUFFER_SIZE];
    let buf_len = messagepack_serde::to_slice(&s, &mut buf).unwrap();

    #[cfg(not(codspeed))]
    {
        bencher = bencher.counter(BytesCount::of_slice(&buf))
    }

    bencher.bench_local(|| {
        let buf = core::hint::black_box(&buf[..buf_len]);
        messagepack_serde::from_slice::<StrTypesBorrowed>(buf).unwrap()
    });
}

#[divan::bench(
    types = [ArrayTypes, ByteType, CompositeType, MapType, PrimitiveTypes, StrTypes],
    consts = LENS
)]
fn deserialize_rmp_serde<T: Serialize + DeserializeOwned + Default + Sync, const N: usize>(
    #[allow(unused_mut)] mut bencher: divan::Bencher,
) {
    let s = repeat_with(|| T::default()).take(N).collect::<Vec<_>>();
    let mut buf = vec![0u8; BUFFER_SIZE * N];
    let buf_len = messagepack_serde::to_slice(&s, &mut buf).unwrap();

    #[cfg(not(codspeed))]
    {
        bencher = bencher.counter(BytesCount::of_slice(&buf))
    }

    bencher.bench_local(|| {
        let buf = core::hint::black_box(&buf[..buf_len]);
        rmp_serde::from_slice::<Vec<T>>(buf).unwrap()
    });
}

#[divan::bench]
fn deserialize_borrowed_rmp_serde(#[allow(unused_mut)] mut bencher: divan::Bencher) {
    let s = StrTypesBorrowed::default();
    let mut buf = vec![0u8; BUFFER_SIZE];
    let buf_len = messagepack_serde::to_slice(&s, &mut buf).unwrap();

    #[cfg(not(codspeed))]
    {
        bencher = bencher.counter(BytesCount::of_slice(&buf))
    }

    bencher.bench_local(|| {
        let buf = core::hint::black_box(&buf[..buf_len]);
        rmp_serde::from_slice::<StrTypesBorrowed>(buf).unwrap()
    });
}
