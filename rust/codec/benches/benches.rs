#[cfg(any(feature = "json", feature = "bincode-serializer"))]
mod datasets;
mod packetizer;
#[cfg(any(feature = "json", feature = "bincode-serializer"))]
mod serializer;

use criterion::{criterion_group, criterion_main, Criterion};

pub fn run(c: &mut Criterion) {
    packetizer::run(c);

    #[cfg(any(feature = "json", feature = "bincode-serializer"))]
    serializer::run(c);
}

criterion_group!(benches, run);
criterion_main!(benches);
