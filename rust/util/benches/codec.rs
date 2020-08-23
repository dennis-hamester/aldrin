mod packetizer;

use criterion::Criterion;

pub fn run(c: &mut Criterion) {
    packetizer::run(c);
}
