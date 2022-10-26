#[cfg(any(feature = "json-serializer", feature = "bincode"))]
mod datasets;
mod packetizer;
#[cfg(any(feature = "json-serializer", feature = "bincode"))]
mod serializer;

use criterion::{criterion_group, criterion_main, Criterion};

pub fn run(c: &mut Criterion) {
    packetizer::run(c);

    #[cfg(any(feature = "json-serializer", feature = "bincode"))]
    serializer::run(c);
}

criterion_group!(benches, run);
criterion_main!(benches);
