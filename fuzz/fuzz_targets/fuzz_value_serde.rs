#![no_main]

use libfuzzer_sys::fuzz_target;
use messagepack_serde::{
    Value, from_slice,
    ser::{Exact, to_vec_with_config},
};

fuzz_target!(|data: &[u8]| {
    // fuzzed code goes here
    let Ok(v) = from_slice::<Value>(data) else {
        return;
    };

    // Since [messagepack_serde::value::Number] doesn't preserve the original type information, encoding it again won't produce the same value.
    // Therefore, we won't perform tests involving re-encoding.
    let _buf = to_vec_with_config(&v, Exact).unwrap();
});
