#![no_main]

use aldrin_core::{ProtocolVersion, SerializedValue, Value};
use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;

#[derive(Debug, Clone, PartialEq, Arbitrary)]
struct Input {
    value: SerializedValue,
    from: Option<ProtocolVersion>,
    to: ProtocolVersion,
}

fuzz_target!(|input: Input| {
    let mut converted = input.value.clone();

    if converted.convert(input.from, input.to).is_err() {
        return;
    }

    let Ok(value) = input.value.deserialize::<Value>() else {
        return;
    };

    let Ok(converted) = converted.deserialize::<Value>() else {
        return;
    };

    // Guard against NaN. We'll miss a lot of checks here, but it's far easier than writing a custom
    // NaN-aware checker.
    if value == value {
        assert_eq!(converted, value);
    }
});
