use aldrin_proto::SerializedValue;
use criterion::{BenchmarkId, Criterion, Throughput};

pub fn serialize(c: &mut Criterion) {
    let mut group = c.benchmark_group("Vec<i32>/serialize");

    for size in [100, 1000, 10000] {
        group.throughput(Throughput::Elements(size));

        let vec: Vec<i32> = (0..size as i32).collect();
        group.bench_with_input(BenchmarkId::from_parameter(size), &vec, |b, vec| {
            b.iter(|| SerializedValue::serialize(vec))
        });
    }

    group.finish();
}

pub fn deserialize(c: &mut Criterion) {
    let mut group = c.benchmark_group("Vec<i32>/deserialize");

    for size in [100, 1000, 10000] {
        group.throughput(Throughput::Elements(size));

        let vec: Vec<i32> = (0..size as i32).collect();
        let vec = SerializedValue::serialize(&vec).unwrap();

        group.bench_with_input(BenchmarkId::from_parameter(size), &vec, |b, vec| {
            b.iter(|| vec.deserialize::<Vec<i32>>())
        });
    }

    group.finish();
}
