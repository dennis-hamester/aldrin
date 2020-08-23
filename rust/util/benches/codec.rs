mod packetizer;
#[cfg(any(feature = "json"))]
mod serializer;

use criterion::Criterion;

pub fn run(c: &mut Criterion) {
    packetizer::run(c);

    #[cfg(any(feature = "json"))]
    serializer::run(c);
}
