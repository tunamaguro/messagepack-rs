#![allow(unexpected_cfgs)]

#[cfg(not(codspeed))]
use divan::counter::BytesCount;
use messagepack_bench::{
    ArrayTypes, BenchData, ByteType, ByteTypeBorrowed, CompositeType, MapType, PrimitiveTypes,
    StrTypes, StrTypesBorrowed,
};
use messagepack_core::Encode;
use serde::Serialize;

#[global_allocator]
static ALLOC: divan::AllocProfiler = divan::AllocProfiler::system();

fn main() {
    // Run registered benchmarks.
    divan::main();
}

const LENS: &[usize] = &[256];
const BUFFER_SIZE: usize = (2u32.pow(18)) as usize;

#[divan::bench(
    types = [ArrayTypes, ByteType, ByteTypeBorrowed, CompositeType, MapType, PrimitiveTypes, StrTypes, StrTypesBorrowed],
    args = LENS
)]
fn serialize_messagepack_serde<T: Serialize + BenchData + Sync>(
    bencher: divan::Bencher,
    len: usize,
) {
    let s = T::generate_vec(len);

    #[allow(unused_mut)]
    let mut bencher = bencher.with_inputs(|| vec![0u8; BUFFER_SIZE * len]);

    #[cfg(not(codspeed))]
    {
        bencher = bencher.input_counter(BytesCount::of_slice);
    }

    bencher.bench_local_refs(|buf| {
        let buf = core::hint::black_box(buf);
        messagepack_serde::to_slice(core::hint::black_box(&s), buf).unwrap()
    });
}

#[divan::bench(
    types = [ArrayTypes, ByteType, ByteTypeBorrowed, CompositeType, MapType, PrimitiveTypes, StrTypes, StrTypesBorrowed],
    args = LENS
)]
fn serialize_rmp_serde<T: Serialize + BenchData + Sync>(bencher: divan::Bencher, len: usize) {
    let s = T::generate_vec(len);

    #[allow(unused_mut)]
    let mut bencher = bencher.with_inputs(|| vec![0u8; BUFFER_SIZE * len]);

    #[cfg(not(codspeed))]
    {
        bencher = bencher.input_counter(BytesCount::of_slice);
    }

    bencher.bench_local_refs(|buf| {
        let buf = core::hint::black_box(buf);
        let mut ser = rmp_serde::Serializer::new(buf).with_struct_map();
        core::hint::black_box(&s).serialize(&mut ser)
    });
}

#[divan::bench(
    types = [ArrayTypes, ByteType, ByteTypeBorrowed, CompositeType, MapType, PrimitiveTypes, StrTypes, StrTypesBorrowed],
    args = LENS
)]
fn serialize_messagepack_core<T: Encode + BenchData + Sync>(bencher: divan::Bencher, len: usize) {
    let s = T::generate_vec(len);

    #[allow(unused_mut)]
    let mut bencher = bencher.with_inputs(|| vec![0u8; BUFFER_SIZE * len]);

    #[cfg(not(codspeed))]
    {
        bencher = bencher.input_counter(BytesCount::of_slice);
    }

    bencher.bench_local_refs(|buf| {
        let buf = core::hint::black_box(buf);
        core::hint::black_box(&s).encode(buf)
    });
}
