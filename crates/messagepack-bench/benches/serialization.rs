use divan::AllocProfiler;
use messagepack_bench::{
    ArrayTypes, ByteType, CompositeType, MapType, PrimitiveTypes, StringTypes,
};
use serde::Serialize;
use std::iter::repeat_with;

#[global_allocator]
static ALLOC: AllocProfiler = AllocProfiler::system();

fn main() {
    // Run registered benchmarks.
    divan::main();
}

const LENS: &[usize] = &[1, 2, 4, 8, 16, 32];
const BUFFER_SIZE: usize = (2u32.pow(16)) as usize;

#[divan::bench(
    types = [ArrayTypes, ByteType, CompositeType, MapType, PrimitiveTypes, StringTypes],
    args = LENS
)]
fn serializer_messagepack_serde<T: Serialize + Default + Sync>(
    bencher: divan::Bencher,
    len: usize,
) {
    let s = repeat_with(|| T::default()).take(len).collect::<Vec<_>>();

    bencher
        .with_inputs(|| vec![0u8; BUFFER_SIZE * len])
        .bench_local_refs(|buf| {
            let buf = core::hint::black_box(buf);
            messagepack_serde::ser::to_slice(core::hint::black_box(&s), buf).unwrap()
        });
}

#[divan::bench(
    types = [ArrayTypes, ByteType, CompositeType, MapType, PrimitiveTypes, StringTypes],
    args = LENS
)]
fn serializer_rmp_serde<T: Serialize + Default + Sync>(bencher: divan::Bencher, len: usize) {
    let s = repeat_with(|| T::default()).take(len).collect::<Vec<_>>();

    bencher
        .with_inputs(|| vec![0u8; BUFFER_SIZE * len])
        .bench_local_refs(|buf| {
            let buf = core::hint::black_box(buf);
            let mut ser = rmp_serde::Serializer::new(buf);
            core::hint::black_box(&s).serialize(&mut ser)
        });
}
