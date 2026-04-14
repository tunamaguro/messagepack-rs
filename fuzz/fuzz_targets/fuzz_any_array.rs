#![no_main]

use libfuzzer_sys::fuzz_target;
use messagepack_core::{
    decode::{Any, Decode},
    io::SliceReader,
};

fuzz_target!(|data: &[u8]| {
    // fuzzed code goes here
    let mut reader = SliceReader::new(data);
    let Ok(_v) = Vec::<Any>::decode(&mut reader) else {
        return;
    };
});
