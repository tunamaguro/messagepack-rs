use divan::AllocProfiler;

#[global_allocator]
static ALLOC: AllocProfiler = AllocProfiler::system();

fn main() {
    // Run registered benchmarks.
    divan::main();
}

mod primtive_type {
    use messagepack_bench::PrimitiveTypes;
    use serde::Serialize;

    #[divan::bench]
    fn messagepack_serde(bencher: divan::Bencher) {
        let s = divan::black_box(rand::random::<PrimitiveTypes>());

        bencher.bench(move || {
            let buf = &mut [0_u8; 4096 * 10];
            messagepack_serde::ser::to_slice(&s, buf).unwrap();
        });
    }

    #[divan::bench]
    fn rmp_serde(bencher: divan::Bencher) {
        let s = divan::black_box(rand::random::<PrimitiveTypes>());

        bencher.bench(move || {
            let mut buf = Vec::with_capacity(4096 * 10);
            let mut ser = rmp_serde::Serializer::new(&mut buf);

            s.serialize(&mut ser).unwrap();
        });
    }
}

mod str_type {
    use messagepack_bench::StringTypes;
    use serde::Serialize;

    #[divan::bench]
    fn messagepack_serde(bencher: divan::Bencher) {
        let s = divan::black_box(StringTypes::default());

        bencher.bench(move || {
            let buf = &mut [0_u8; 4096 * 10];
            messagepack_serde::ser::to_slice(&s, buf).unwrap();
        });
    }

    #[divan::bench]
    fn rmp_serde(bencher: divan::Bencher) {
        let s = divan::black_box(StringTypes::default());

        bencher.bench(move || {
            let mut buf = Vec::with_capacity(4096 * 10);
            let mut ser = rmp_serde::Serializer::new(&mut buf);

            s.serialize(&mut ser).unwrap();
        });
    }
}

mod array_type {
    use messagepack_bench::ArrayTypes;
    use serde::Serialize;

    #[divan::bench]
    fn messagepack_serde(bencher: divan::Bencher) {
        let s = divan::black_box(ArrayTypes::default());

        bencher.bench(move || {
            let buf = &mut [0_u8; 4096 * 10];
            messagepack_serde::ser::to_slice(&s, buf).unwrap();
        });
    }

    #[divan::bench]
    fn rmp_serde(bencher: divan::Bencher) {
        let s = divan::black_box(ArrayTypes::default());

        bencher.bench(move || {
            let mut buf = Vec::with_capacity(4096 * 10);
            let mut ser = rmp_serde::Serializer::new(&mut buf);

            s.serialize(&mut ser).unwrap();
        });
    }
}

mod byte_type {
    use messagepack_bench::ByteType;
    use serde::Serialize;

    #[divan::bench]
    fn messagepack_serde(bencher: divan::Bencher) {
        let s = divan::black_box(ByteType::default());

        bencher.bench(move || {
            let buf = &mut [0_u8; 4096 * 10];
            messagepack_serde::ser::to_slice(&s, buf).unwrap();
        });
    }

    #[divan::bench]
    fn rmp_serde(bencher: divan::Bencher) {
        let s = divan::black_box(ByteType::default());

        bencher.bench(move || {
            let mut buf = Vec::with_capacity(4096 * 10);
            let mut ser = rmp_serde::Serializer::new(&mut buf);

            s.serialize(&mut ser).unwrap();
        });
    }
}

mod composite_type {
    use messagepack_bench::CompositeType;
    use serde::Serialize;

    #[divan::bench]
    fn messagepack_serde(bencher: divan::Bencher) {
        let s = divan::black_box(CompositeType::default());

        bencher.bench(move || {
            let buf = &mut [0_u8; 4096 * 10];
            messagepack_serde::ser::to_slice(&s, buf).unwrap();
        });
    }

    #[divan::bench]
    fn rmp_serde(bencher: divan::Bencher) {
        let s = divan::black_box(CompositeType::default());

        bencher.bench(move || {
            let mut buf = Vec::with_capacity(4096 * 10);
            let mut ser = rmp_serde::Serializer::new(&mut buf);

            s.serialize(&mut ser).unwrap();
        });
    }
}
