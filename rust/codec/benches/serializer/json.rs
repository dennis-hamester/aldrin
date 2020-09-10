use crate::datasets::{MessageSize, Messages};
use aldrin_codec::serializer::Json;
use aldrin_codec::Serializer;
use criterion::measurement::Measurement;
use criterion::{BatchSize, BenchmarkGroup, BenchmarkId};
use std::fmt;

pub fn serialize<M: Measurement>(
    group: &mut BenchmarkGroup<M>,
    dataset: &Messages,
    size: MessageSize,
) {
    for &pretty in &[true, false] {
        let input = JsonInput { size, pretty };
        group.bench_with_input(
            BenchmarkId::new("JSON/serialize", input),
            &input,
            |b, input| {
                b.iter_batched(
                    || {
                        let json = Json::with_pretty(input.pretty);
                        let msg = dataset.get(input.size).clone();
                        (json, msg)
                    },
                    |(mut json, msg)| json.serialize(msg).unwrap(),
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
    for &pretty in &[true, false] {
        let input = JsonInput { size, pretty };
        group.bench_with_input(
            BenchmarkId::new("JSON/deserialize", input),
            &input,
            |b, input| {
                b.iter_batched(
                    || {
                        let mut json = Json::with_pretty(input.pretty);
                        let msg = dataset.get(input.size).clone();
                        let data = json.serialize(msg).unwrap().freeze();
                        (json, data)
                    },
                    |(mut json, data)| json.deserialize(data).unwrap(),
                    BatchSize::SmallInput,
                );
            },
        );
    }
}

#[derive(Copy, Clone)]
struct JsonInput {
    size: MessageSize,
    pretty: bool,
}

impl fmt::Display for JsonInput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let pretty = if self.pretty { "pretty" } else { "not pretty" };
        write!(f, "{}/{}", self.size, pretty)
    }
}
