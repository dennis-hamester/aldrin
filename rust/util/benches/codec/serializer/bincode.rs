use crate::datasets::{MessageSize, Messages};
use aldrin_util::codec::{BincodeSerializer, Endian, Serializer};
use criterion::measurement::Measurement;
use criterion::{BatchSize, BenchmarkGroup, BenchmarkId};
use std::fmt;

pub fn serialize<M: Measurement>(
    group: &mut BenchmarkGroup<M>,
    dataset: &Messages,
    size: MessageSize,
) {
    for &endian in &[Endian::Big, Endian::Little] {
        let input = BincodeInput { size, endian };
        group.bench_with_input(
            BenchmarkId::new("Bincode/serialize", input),
            &input,
            |b, input| {
                b.iter_batched(
                    || {
                        let bincode = BincodeSerializer::with_endian(input.endian);
                        let msg = dataset.get(input.size).clone();
                        (bincode, msg)
                    },
                    |(mut bincode, msg)| bincode.serialize(msg).unwrap(),
                    BatchSize::SmallInput,
                );
            },
        );
    }
}

pub fn deserialize<M: Measurement>(
    group: &mut BenchmarkGroup<M>,
    dataset: &Messages,
    size: MessageSize,
) {
    for &endian in &[Endian::Big, Endian::Little] {
        let input = BincodeInput { size, endian };
        group.bench_with_input(
            BenchmarkId::new("Bincode/deserialize", input),
            &input,
            |b, input| {
                b.iter_batched(
                    || {
                        let mut bincode = BincodeSerializer::with_endian(input.endian);
                        let msg = dataset.get(input.size).clone();
                        let data = bincode.serialize(msg).unwrap().freeze();
                        (bincode, data)
                    },
                    |(mut bincode, data)| bincode.deserialize(data).unwrap(),
                    BatchSize::SmallInput,
                );
            },
        );
    }
}

#[derive(Copy, Clone)]
struct BincodeInput {
    size: MessageSize,
    endian: Endian,
}

impl fmt::Display for BincodeInput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let endian = match self.endian {
            Endian::Big => "big",
            Endian::Little => "little",
        };
        write!(f, "{}/{} endian", self.size, endian)
    }
}
