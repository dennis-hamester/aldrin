#![no_main]

use aldrin_core::{Enum, ProtocolVersion, SerializedValue, Value};
use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use std::collections::HashMap;
use std::hash::Hash;

#[derive(Debug, Clone, PartialEq, Arbitrary)]
struct Input {
    value: SerializedValue,
    from: Option<ProtocolVersion>,
    to: ProtocolVersion,
}

fuzz_target!(|input: Input| {
    let mut converted = input.value.clone();

    if converted.convert(input.from, input.to).is_err() {
        return;
    }

    let Ok(value) = input.value.deserialize() else {
        return;
    };

    let converted = converted.deserialize().unwrap();

    assert!(value_eq(&value, &converted));
});

fn value_eq(a: &Value, b: &Value) -> bool {
    let mut stack = vec![(a, b)];

    while let Some((a, b)) = stack.pop() {
        let eq = match (a, b) {
            (Value::None, Value::None) => true,

            (Value::Some(a), Value::Some(b)) => {
                stack.push((a, b));
                true
            }

            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::U8(a), Value::U8(b)) => a == b,
            (Value::I8(a), Value::I8(b)) => a == b,
            (Value::U16(a), Value::U16(b)) => a == b,
            (Value::I16(a), Value::I16(b)) => a == b,
            (Value::U32(a), Value::U32(b)) => a == b,
            (Value::I32(a), Value::I32(b)) => a == b,
            (Value::U64(a), Value::U64(b)) => a == b,
            (Value::I64(a), Value::I64(b)) => a == b,
            (Value::F32(a), Value::F32(b)) => f32_eq(*a, *b),
            (Value::F64(a), Value::F64(b)) => f64_eq(*a, *b),
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Uuid(a), Value::Uuid(b)) => a == b,
            (Value::ObjectId(a), Value::ObjectId(b)) => a == b,
            (Value::ServiceId(a), Value::ServiceId(b)) => a == b,
            (Value::Vec(a), Value::Vec(b)) => vec_eq(a, b, &mut stack),
            (Value::Bytes(a), Value::Bytes(b)) => a == b,
            (Value::U8Map(a), Value::U8Map(b)) => map_eq(a, b, &mut stack),
            (Value::I8Map(a), Value::I8Map(b)) => map_eq(a, b, &mut stack),
            (Value::U16Map(a), Value::U16Map(b)) => map_eq(a, b, &mut stack),
            (Value::I16Map(a), Value::I16Map(b)) => map_eq(a, b, &mut stack),
            (Value::U32Map(a), Value::U32Map(b)) => map_eq(a, b, &mut stack),
            (Value::I32Map(a), Value::I32Map(b)) => map_eq(a, b, &mut stack),
            (Value::U64Map(a), Value::U64Map(b)) => map_eq(a, b, &mut stack),
            (Value::I64Map(a), Value::I64Map(b)) => map_eq(a, b, &mut stack),
            (Value::StringMap(a), Value::StringMap(b)) => map_eq(a, b, &mut stack),
            (Value::UuidMap(a), Value::UuidMap(b)) => map_eq(a, b, &mut stack),
            (Value::U8Set(a), Value::U8Set(b)) => a == b,
            (Value::I8Set(a), Value::I8Set(b)) => a == b,
            (Value::U16Set(a), Value::U16Set(b)) => a == b,
            (Value::I16Set(a), Value::I16Set(b)) => a == b,
            (Value::U32Set(a), Value::U32Set(b)) => a == b,
            (Value::I32Set(a), Value::I32Set(b)) => a == b,
            (Value::U64Set(a), Value::U64Set(b)) => a == b,
            (Value::I64Set(a), Value::I64Set(b)) => a == b,
            (Value::StringSet(a), Value::StringSet(b)) => a == b,
            (Value::UuidSet(a), Value::UuidSet(b)) => a == b,
            (Value::Struct(a), Value::Struct(b)) => map_eq(&a.0, &b.0, &mut stack),
            (Value::Enum(a), Value::Enum(b)) => enum_eq(a, b, &mut stack),
            (Value::Sender(a), Value::Sender(b)) => a == b,
            (Value::Receiver(a), Value::Receiver(b)) => a == b,
            _ => false,
        };

        if !eq {
            return false;
        }
    }

    true
}

fn f32_eq(a: f32, b: f32) -> bool {
    a.to_bits() == b.to_bits()
}

fn f64_eq(a: f64, b: f64) -> bool {
    a.to_bits() == b.to_bits()
}

fn vec_eq<'a>(a: &'a [Value], b: &'a [Value], stack: &mut Vec<(&'a Value, &'a Value)>) -> bool {
    if a.len() == b.len() {
        stack.extend(a.iter().zip(b));
        true
    } else {
        false
    }
}

fn map_eq<'a, T: Eq + Hash>(
    a: &'a HashMap<T, Value>,
    b: &'a HashMap<T, Value>,
    stack: &mut Vec<(&'a Value, &'a Value)>,
) -> bool {
    if a.len() == b.len() {
        a.iter().all(|(k, a)| {
            b.get(k).is_some_and(|b| {
                stack.push((a, b));
                true
            })
        })
    } else {
        false
    }
}

fn enum_eq<'a>(a: &'a Enum, b: &'a Enum, stack: &mut Vec<(&'a Value, &'a Value)>) -> bool {
    if a.id == b.id {
        stack.push((&a.value, &b.value));
        true
    } else {
        false
    }
}
