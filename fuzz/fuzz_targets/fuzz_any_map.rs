#![no_main]

use libfuzzer_sys::fuzz_target;
use messagepack_core::{
    decode::{Any, Decode, MapDecoder},
    io::SliceReader,
};

fuzz_target!(|data: &[u8]| {
    // fuzzed code goes here
    let mut reader = SliceReader::new(data);
    let Ok(_v) = MapDecoder::<Vec<(Any, Any)>, Any, Any>::decode(&mut reader) else {
        return;
    };
});
