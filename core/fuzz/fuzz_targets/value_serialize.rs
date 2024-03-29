#![no_main]

use aldrin_core::{SerializedValue, Value};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|value: Value| {
    let _ = SerializedValue::serialize(&value);
});
