#[cfg(feature = "codec")]
mod codec;
#[cfg(any(feature = "json", feature = "bincode-serializer"))]
mod datasets;

use criterion::{criterion_group, criterion_main, Criterion};

#[allow(unused_variables)]
pub fn run(c: &mut Criterion) {
    #[cfg(feature = "codec")]
    codec::run(c);
}

criterion_group!(benches, run);
criterion_main!(benches);
