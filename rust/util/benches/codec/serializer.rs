#[cfg(feature = "json")]
mod json;

use aldrin_proto::{CallFunction, CallFunctionReply, CallFunctionResult, Message, Value};
use criterion::{Criterion, Throughput};
use maplit::hashmap;
use std::fmt;
use uuid::Uuid;

const UUID1: Uuid = Uuid::from_u128(0xdc9500bd922647b2a5c8e6badccaf0c4);
const UUID2: Uuid = Uuid::from_u128(0x06b5584580154c0ab24834fee7a98edf);

pub fn run(c: &mut Criterion) {
    let mut group = c.benchmark_group("Serializers");
    for &size in &[MessageSize::Small, MessageSize::Large] {
        group.throughput(Throughput::Elements(1));

        #[cfg(feature = "json")]
        {
            json::serialize(&mut group, size);
            json::deserialize(&mut group, size);
        }
    }
    group.finish();
}

#[derive(Copy, Clone)]
pub enum MessageSize {
    Small,
    Large,
}

impl MessageSize {
    fn get(self) -> Message {
        match self {
            MessageSize::Small => Self::small(),
            MessageSize::Large => Self::large(),
        }
    }

    fn small() -> Message {
        Message::CallFunctionReply(CallFunctionReply {
            serial: 1,
            result: CallFunctionResult::Ok(Value::None),
        })
    }

    fn large() -> Message {
        Message::CallFunction(CallFunction {
            serial: 1,
            service_cookie: UUID1,
            function: 2,
            args: Value::Struct(hashmap! {
                1 => Value::String("foobar".to_owned()),
                2 => Value::Vec(vec![
                    Value::U64(0x1234567890abcdef),
                    Value::Enum {
                        variant: 1000,
                        value: Box::new(Value::None),
                    },
                ]),
                3 => Value::I32(-10000),
                4 => Value::Uuid(UUID2),
                5 => Value::Bytes(vec![0; 64]),
            }),
        })
    }
}

impl fmt::Display for MessageSize {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let size = match self {
            MessageSize::Small => "small",
            MessageSize::Large => "large",
        };
        f.write_str(size)
    }
}
