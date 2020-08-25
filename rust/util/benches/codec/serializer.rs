#[cfg(feature = "bincode-serializer")]
mod bincode;
#[cfg(feature = "json")]
mod json;

use crate::datasets::{MessageSize, Messages};
use criterion::{Criterion, Throughput};

pub fn run(c: &mut Criterion) {
    let mut group = c.benchmark_group("Serializers");
    let dataset = Messages::open();
    for &size in &[MessageSize::Small, MessageSize::Large] {
        group.throughput(Throughput::Elements(1));

        #[cfg(feature = "json")]
        {
            json::serialize(&mut group, &dataset, size);
            json::deserialize(&mut group, &dataset, size);
        }

        #[cfg(feature = "bincode-serializer")]
        {
            bincode::serialize(&mut group, &dataset, size);
            bincode::deserialize(&mut group, &dataset, size);
        }
    }
    group.finish();
}
