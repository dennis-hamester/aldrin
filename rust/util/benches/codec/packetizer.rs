use aldrin_util::codec::{Endian, LengthPrefixed, Packetizer};
use bytes::{Bytes, BytesMut};
use criterion::measurement::Measurement;
use criterion::{BatchSize, BenchmarkGroup, BenchmarkId, Criterion, Throughput};
use std::fmt;

const SIZES: &[usize] = &[256, 1024, 4096];

pub fn run(c: &mut Criterion) {
    let mut group = c.benchmark_group("Packetizers");
    for &size in SIZES {
        group.throughput(Throughput::Bytes(size as u64));
        length_prefixed_encode(&mut group, size);
        length_prefixed_decode(&mut group, size);
    }
    group.finish();
}

fn length_prefixed_encode<M: Measurement>(group: &mut BenchmarkGroup<M>, size: usize) {
    for &endian in &[Endian::Big, Endian::Little] {
        let input = LengthPrefixedInput { size, endian };
        group.bench_with_input(
            BenchmarkId::new("LengthPrefixed/encode", input),
            &input,
            |b, input| {
                b.iter_batched(
                    || {
                        let length_prefixed =
                            LengthPrefixed::builder().endian(input.endian).build();
                        let data = create_data(input.size);
                        let dst = BytesMut::with_capacity(2 * data.len());
                        (length_prefixed, data, dst)
                    },
                    |(mut length_prefixed, data, mut dst)| {
                        length_prefixed.encode(data, &mut dst).unwrap();
                        dst
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }
}

fn length_prefixed_decode<M: Measurement>(group: &mut BenchmarkGroup<M>, size: usize) {
    for &endian in &[Endian::Big, Endian::Little] {
        let input = LengthPrefixedInput { size, endian };
        group.bench_with_input(
            BenchmarkId::new("LengthPrefixed/decode", input),
            &input,
            |b, input| {
                b.iter_batched(
                    || {
                        let mut length_prefixed =
                            LengthPrefixed::builder().endian(input.endian).build();
                        let data = create_data(input.size);
                        let mut encoded = BytesMut::new();
                        length_prefixed.encode(data, &mut encoded).unwrap();
                        (length_prefixed, encoded)
                    },
                    |(mut length_prefixed, mut data)| {
                        length_prefixed.decode(&mut data).unwrap().unwrap()
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }
}

fn create_data(size: usize) -> Bytes {
    Bytes::from(vec![0; size])
}

#[derive(Copy, Clone)]
struct LengthPrefixedInput {
    size: usize,
    endian: Endian,
}

impl fmt::Display for LengthPrefixedInput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let endian = match self.endian {
            Endian::Big => "big",
            Endian::Little => "little",
        };
        write!(f, "{} bytes/{} endian", self.size, endian)
    }
}
