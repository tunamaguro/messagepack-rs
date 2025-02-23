use divan::AllocProfiler;

#[global_allocator]
static ALLOC: AllocProfiler = AllocProfiler::system();

fn main() {
    // Run registered benchmarks.
    divan::main();
}

#[divan::bench_group]
mod primtive_type {
    use messagepack_bench::PrimitiveTypes;
    use serde::Serialize;

    #[divan::bench(name = "messagepack_serde_primitive")]
    fn messagepack_serde_(bencher: divan::Bencher) {
        let s = divan::black_box(rand::random::<PrimitiveTypes>());

        bencher.bench(move || {
            let buf = &mut [0_u8; 4096 * 10];
            messagepack_serde::ser::to_slice(&s, divan::black_box(buf)).unwrap();
        });
    }

    #[divan::bench(name = "rmp_serde_primitive")]
    fn rmp_serde_(bencher: divan::Bencher) {
        let s = divan::black_box(rand::random::<PrimitiveTypes>());

        bencher.bench(move || {
            let mut buf = Vec::with_capacity(4096 * 10);
            let mut ser = rmp_serde::Serializer::new(divan::black_box(&mut buf));

            s.serialize(&mut ser).unwrap();
        });
    }
}

#[divan::bench_group]
mod str_type {
    use messagepack_bench::StringTypes;
    use serde::Serialize;

    #[divan::bench(name = "messagepack_serde_str")]
    fn messagepack_serde(bencher: divan::Bencher) {
        let s = divan::black_box(StringTypes::default());

        bencher.bench(move || {
            let buf = &mut [0_u8; 4096 * 10];
            messagepack_serde::ser::to_slice(&s, divan::black_box(buf)).unwrap();
        });
    }

    #[divan::bench(name = "rmp_serde_str")]
    fn rmp_serde(bencher: divan::Bencher) {
        let s = divan::black_box(StringTypes::default());

        bencher.bench(move || {
            let mut buf = Vec::with_capacity(4096 * 10);
            let mut ser = rmp_serde::Serializer::new(divan::black_box(&mut buf));

            s.serialize(&mut ser).unwrap();
        });
    }
}

#[divan::bench_group]
mod array_type {
    use messagepack_bench::ArrayTypes;
    use serde::Serialize;

    #[divan::bench(name = "messagepack_serde_array")]
    fn messagepack_serde(bencher: divan::Bencher) {
        let s = divan::black_box(ArrayTypes::default());

        bencher.bench(move || {
            let buf = &mut [0_u8; 4096 * 10];
            messagepack_serde::ser::to_slice(&s, divan::black_box(buf)).unwrap();
        });
    }

    #[divan::bench(name = "rmp_serde_array")]
    fn rmp_serde(bencher: divan::Bencher) {
        let s = divan::black_box(ArrayTypes::default());

        bencher.bench(move || {
            let mut buf = Vec::with_capacity(4096 * 10);
            let mut ser = rmp_serde::Serializer::new(divan::black_box(&mut buf));

            s.serialize(&mut ser).unwrap();
        });
    }
}

#[divan::bench_group]
mod byte_type {
    use messagepack_bench::ByteType;
    use serde::Serialize;

    #[divan::bench(name = "messagepack_serde_byte")]
    fn messagepack_serde(bencher: divan::Bencher) {
        let s = divan::black_box(ByteType::default());

        bencher.bench(move || {
            let buf = &mut [0_u8; 4096 * 10];
            messagepack_serde::ser::to_slice(&s, divan::black_box(buf)).unwrap();
        });
    }

    #[divan::bench(name = "rmp_serde_byte")]
    fn rmp_serde(bencher: divan::Bencher) {
        let s = divan::black_box(ByteType::default());

        bencher.bench(move || {
            let mut buf = Vec::with_capacity(4096 * 10);
            let mut ser = rmp_serde::Serializer::new(divan::black_box(&mut buf));

            s.serialize(&mut ser).unwrap();
        });
    }
}

#[divan::bench_group]
mod composite_type {
    use messagepack_bench::CompositeType;
    use serde::Serialize;

    #[divan::bench(name = "messagepack_serde_composite")]
    fn messagepack_serde(bencher: divan::Bencher) {
        let s = divan::black_box(CompositeType::default());

        bencher.bench(move || {
            let buf = &mut [0_u8; 4096 * 10];
            messagepack_serde::ser::to_slice(&s, divan::black_box(buf)).unwrap();
        });
    }

    #[divan::bench(name = "rmp_serde_composite")]
    fn rmp_serde(bencher: divan::Bencher) {
        let s = divan::black_box(CompositeType::default());

        bencher.bench(move || {
            let mut buf = Vec::with_capacity(4096 * 10);
            let mut ser = rmp_serde::Serializer::new(divan::black_box(&mut buf));

            s.serialize(&mut ser).unwrap();
        });
    }
}
