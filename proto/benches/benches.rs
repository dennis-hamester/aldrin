mod all_messages;
mod big_value;
mod vec_i32;

use criterion::{criterion_group, criterion_main};

criterion_group!(
    benches,
    all_messages::deserialize,
    all_messages::serialize,
    big_value::deserialize,
    big_value::serialize,
    vec_i32::deserialize,
    vec_i32::serialize,
);

criterion_main!(benches);
