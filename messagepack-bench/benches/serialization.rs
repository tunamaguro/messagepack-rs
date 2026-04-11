#![allow(unexpected_cfgs)]

#[cfg(not(codspeed))]
use divan::counter::BytesCount;
use messagepack_bench::{
    ArrayTypes, ByteType, ByteTypeBorrowed, CompositeType, MapType, PrimitiveTypes, StrTypes,
    StrTypesBorrowed,
};
use serde::Serialize;
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
    types = [ArrayTypes, ByteType, ByteTypeBorrowed, CompositeType, MapType, PrimitiveTypes, StrTypes, StrTypesBorrowed],
    args = LENS
)]
fn serialize_messagepack_serde<T: Serialize + Default + Sync>(bencher: divan::Bencher, len: usize) {
    let s = repeat_with(|| T::default()).take(len).collect::<Vec<_>>();

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
fn serialize_rmp_serde<T: Serialize + Default + Sync>(bencher: divan::Bencher, len: usize) {
    let s = repeat_with(|| T::default()).take(len).collect::<Vec<_>>();

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
