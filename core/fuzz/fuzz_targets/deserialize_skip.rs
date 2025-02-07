#![no_main]

use aldrin_core::{tags, Deserialize, DeserializeError, Deserializer, SerializedValue};
use libfuzzer_sys::fuzz_target;

struct Skip;

impl Deserialize<tags::Value> for Skip {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.skip().map(|()| Self)
    }
}

fuzz_target!(|value: SerializedValue| {
    let _ = value.deserialize_as::<tags::Value, Skip>();
});
