mod big_value;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn run(c: &mut Criterion) {}

criterion_group!(benches, run);
criterion_main!(benches);
