use divan::AllocProfiler;

#[global_allocator]
static ALLOC: AllocProfiler = AllocProfiler::system();

fn main() {
    // Run registered benchmarks.
    divan::main();
}

mod str_type {
    use messagepack_bench::StringTypes;
    use serde::Serialize;

    #[divan::bench]
    fn messagepack_serde_() {
        let s = StringTypes::default();
        let buf = &mut [0_u8; 4096 * 10];

        messagepack_serde::ser::to_slice(&s, buf).unwrap();
    }

    #[divan::bench]
    fn rmp_serde_() {
        let s = StringTypes::default();
        let mut buf = Vec::with_capacity(4096 * 10);
        let mut ser = rmp_serde::Serializer::new(&mut buf);

        s.serialize(&mut ser).unwrap();
    }
}
