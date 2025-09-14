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
    args = LENS
)]
fn deserialize_messagepack_serde<T: Serialize + DeserializeOwned + Default + Sync>(
    #[allow(unused_mut)] mut bencher: divan::Bencher,
    len: usize,
) {
    let s = repeat_with(|| T::default()).take(len).collect::<Vec<_>>();
    let mut buf = vec![0u8; BUFFER_SIZE * len];
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
    args = LENS
)]
fn deserialize_rmp_serde<T: Serialize + DeserializeOwned + Default + Sync>(
    #[allow(unused_mut)] mut bencher: divan::Bencher,
    len: usize,
) {
    let s = repeat_with(|| T::default()).take(len).collect::<Vec<_>>();
    let mut buf = vec![0u8; BUFFER_SIZE * len];
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

// Copy from https://github.com/3Hren/msgpack-rust/blob/09eade54a06e68273ad438a2b9bcbbc08ea6f4dc/rmpv-tests/benches/value.rs
const COMPLEX: &[u8] = &[
    0x94, 0x01, 0x00, 0x93, 0x91, 0x92, 0xa9, 0x31, 0x32, 0x37, 0x2e, 0x30, 0x2e, 0x30, 0x2e, 0x31,
    0xcd, 0xe6, 0xc2, 0x01, 0x84, 0x00, 0x93, 0xa4, 0x72, 0x65, 0x61, 0x64, 0x80, 0x82, 0x00, 0x92,
    0xa5, 0x76, 0x61, 0x6c, 0x75, 0x65, 0x80, 0x01, 0x92, 0xa5, 0x65, 0x72, 0x72, 0x6f, 0x72, 0x80,
    0x01, 0x93, 0xa5, 0x77, 0x72, 0x69, 0x74, 0x65, 0x80, 0x82, 0x00, 0x92, 0xa5, 0x76, 0x61, 0x6c,
    0x75, 0x65, 0x80, 0x01, 0x92, 0xa5, 0x65, 0x72, 0x72, 0x6f, 0x72, 0x80, 0x02, 0x93, 0xa6, 0x72,
    0x65, 0x6d, 0x6f, 0x76, 0x65, 0x80, 0x82, 0x00, 0x92, 0xa5, 0x76, 0x61, 0x6c, 0x75, 0x65, 0x80,
    0x01, 0x92, 0xa5, 0x65, 0x72, 0x72, 0x6f, 0x72, 0x80, 0x03, 0x93, 0xa4, 0x66, 0x69, 0x6e, 0x64,
    0x80, 0x82, 0x00, 0x92, 0xa5, 0x76, 0x61, 0x6c, 0x75, 0x65, 0x80, 0x01, 0x92, 0xa5, 0x65, 0x72,
    0x72, 0x6f, 0x72, 0x80, 0x91, 0x93, 0x50, 0x51, 0x52,
];

#[divan::bench]
fn deserialize_complex_messagepack_serde_from_slice(
    #[allow(unused_mut)] mut bencher: divan::Bencher,
) {
    use messagepack_serde::{ValueRef, from_slice};

    #[cfg(not(codspeed))]
    {
        bencher = bencher.counter(BytesCount::of_slice(&COMPLEX))
    }

    bencher.bench_local(|| {
        let input = core::hint::black_box(COMPLEX);
        let _val = from_slice::<ValueRef<'_>>(input).unwrap();
    });
}

#[divan::bench]
fn deserialize_complex_rmp_serde_from_slice(#[allow(unused_mut)] mut bencher: divan::Bencher) {
    use rmp_serde::from_slice;
    use rmpv::ValueRef;

    #[cfg(not(codspeed))]
    {
        bencher = bencher.counter(BytesCount::of_slice(&COMPLEX))
    }

    bencher.bench_local(|| {
        let input = core::hint::black_box(COMPLEX);
        let _val: ValueRef<'_> = from_slice(input).unwrap();
    });
}

// #[divan::bench]
// fn deserialize_complex_messagepack_serde_from_reader(
//     #[allow(unused_mut)] mut bencher: divan::Bencher,
// ) {
//     use messagepack_serde::{Value, from_reader};

//     #[cfg(not(codspeed))]
//     {
//         bencher = bencher.counter(BytesCount::of_slice(&COMPLEX))
//     }

//     bencher.bench_local(|| {
//         let input = core::hint::black_box(std::io::Cursor::new(COMPLEX));
//         let _val: Value = from_reader(input).unwrap();
//     });
// }

// #[divan::bench]
// fn deserialize_complex_rmp_serde_from_reader(#[allow(unused_mut)] mut bencher: divan::Bencher) {
//     use rmp_serde::from_read;
//     use rmpv::Value;

//     #[cfg(not(codspeed))]
//     {
//         bencher = bencher.counter(BytesCount::of_slice(&COMPLEX))
//     }

//     bencher.bench_local(|| {
//         let input = core::hint::black_box(std::io::Cursor::new(COMPLEX));
//         let _val: Value = from_read(input).unwrap();
//     });
// }
