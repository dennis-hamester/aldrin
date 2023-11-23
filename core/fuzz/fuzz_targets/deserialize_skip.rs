#![no_main]

use aldrin_core::{SerializedValue, Skip};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|value: SerializedValue| {
    let _ = value.deserialize::<Skip>();
});
