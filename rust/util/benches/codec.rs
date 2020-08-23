mod packetizer;
#[cfg(any(feature = "json", feature = "bincode-serializer"))]
mod serializer;

use criterion::Criterion;

pub fn run(c: &mut Criterion) {
    packetizer::run(c);

    #[cfg(any(feature = "json", feature = "bincode-serializer"))]
    serializer::run(c);
}
