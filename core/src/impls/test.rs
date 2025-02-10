use crate::{
    BusListenerCookie, ChannelCookie, Deserialize, Deserializer, ObjectCookie, ObjectId,
    ObjectUuid, Receiver, Sender, Serialize, SerializedValue, SerializedValueSlice, ServiceCookie,
    ServiceId, ServiceUuid, Tag, TypeId, Value,
};
use std::collections::{LinkedList, VecDeque};
use std::fmt::Debug;
use std::{f32, f64};
use uuid::{uuid, Uuid};

#[track_caller]
fn assert_serialize<'a, T, U, B>(value: &'a U, expected: B)
where
    T: Tag,
    U: Serialize<T> + Clone,
    &'a U: Serialize<T>,
    B: AsRef<[u8]>,
{
    let serialized_as_t = SerializedValue::serialize::<T, U>(value.clone()).unwrap();
    assert_eq!(serialized_as_t[..], *expected.as_ref());

    let serialized_as_t = SerializedValue::serialize::<T, &U>(value).unwrap();
    assert_eq!(serialized_as_t[..], *expected.as_ref());
}

#[track_caller]
fn assert_serialize_with_value<'a, T, U, B>(value: &'a U, expected: B)
where
    T: Tag,
    U: Serialize<T> + Serialize<Value> + Clone,
    &'a U: Serialize<T> + Serialize<Value>,
    B: AsRef<[u8]>,
{
    assert_serialize::<T, U, _>(value, &expected);

    let serialized_as_value = SerializedValue::serialize::<Value, _>(value.clone()).unwrap();
    assert_eq!(serialized_as_value[..], *expected.as_ref());
}

#[track_caller]
fn assert_deserialize<T, U, B>(expected: &U, serialized: B)
where
    T: Tag,
    U: Deserialize<T> + PartialEq + Debug,
    B: AsRef<[u8]>,
{
    let serialized_value = SerializedValueSlice::new(&serialized);

    // Actual deserialization
    assert_eq!(*expected, serialized_value.deserialize().unwrap());

    // skip
    let mut buf = serialized.as_ref();
    Deserializer::new(&mut buf, 0).unwrap().skip().unwrap();
    assert_eq!(*buf, []);
    // assert_eq!(serialized_value.deserialize(), Ok(Skip));

    // // len
    // let mut buf = serialized.as_ref();
    // let len = Deserializer::new(&mut buf, 0).unwrap().len().unwrap();
    // assert_eq!(len, buf.len());
}

#[track_caller]
fn assert_deserialize_with_value<T, U, B>(expected: &U, serialized: B)
where
    T: Tag,
    U: Deserialize<T> + Deserialize<Value> + PartialEq + Debug,
    B: AsRef<[u8]>,
{
    assert_deserialize::<T, _, _>(expected, &serialized);

    let serialized_value = SerializedValueSlice::new(&serialized);

    assert_eq!(
        *expected,
        serialized_value.deserialize::<Value, _>().unwrap()
    );
}

#[track_caller]
fn assert_serde<'a, T, U, B>(value: &'a U, serialized: B)
where
    T: Tag,
    U: Serialize<T> + Deserialize<T> + Clone + PartialEq + Debug,
    &'a U: Serialize<T>,
    B: AsRef<[u8]>,
{
    assert_serialize::<T, U, _>(value, &serialized);
    assert_deserialize::<T, U, _>(value, &serialized);
}

#[track_caller]
fn assert_serde_with_value<'a, T, U, B>(value: &'a U, serialized: B)
where
    T: Tag,
    U: Serialize<T>
        + Serialize<Value>
        + Deserialize<T>
        + Deserialize<Value>
        + Clone
        + PartialEq
        + Debug,
    &'a U: Serialize<T> + Serialize<Value>,
    B: AsRef<[u8]>,
{
    assert_serialize_with_value::<T, U, _>(value, &serialized);
    assert_deserialize_with_value::<T, U, _>(value, &serialized);
}

#[test]
fn test_unit() {
    type Tag = ();
    let serialized = [0];

    let value = ();
    assert_serialize_with_value::<Tag, (), _>(&value, serialized);
    assert_deserialize_with_value::<Tag, _, _>(&value, serialized);

    let value = Value::None;
    assert_serialize_with_value::<Tag, Value, _>(&value, serialized);
    assert_deserialize_with_value::<Tag, _, _>(&value, serialized);
}

#[test]
fn test_option_none() {
    type Tag = Option<()>;
    let serialized = [0];

    let value = None::<()>;
    assert_serialize_with_value::<Tag, Option<()>, _>(&value, serialized);
    assert_deserialize_with_value::<Tag, _, _>(&value, serialized);

    let value = Value::None;
    assert_serialize_with_value::<Tag, Value, _>(&value, serialized);
    assert_deserialize_with_value::<Tag, _, _>(&value, serialized);

    let value = ();
    assert_serialize_with_value::<Tag, (), _>(&value, serialized);
    assert_deserialize_with_value::<Tag, _, _>(&value, serialized);
}

#[test]
fn test_option_some() {
    type Tag = Option<()>;
    let serialized = [1, 0];

    let value = Some(());
    assert_serde_with_value::<Tag, Option<()>, _>(&value, serialized);

    let value = Value::Some(Box::new(Value::None));
    assert_serde_with_value::<Tag, Value, _>(&value, serialized);
}

#[test]
fn test_bool_false() {
    type Tag = bool;
    let serialized = [2, 0];

    let value = false;
    assert_serde_with_value::<Tag, bool, _>(&value, serialized);

    let value = Value::Bool(false);
    assert_serde_with_value::<Tag, Value, _>(&value, serialized);
}

#[test]
fn test_bool_true() {
    type Tag = bool;
    let serialized = [2, 1];

    let value = true;
    assert_serde_with_value::<Tag, bool, _>(&value, serialized);

    let value = Value::Bool(true);
    assert_serde_with_value::<Tag, Value, _>(&value, serialized);
}

#[test]
fn test_bool_non_zero() {
    type Tag = bool;
    let serialized = [2, 2];

    let value = true;
    assert_deserialize_with_value::<Tag, bool, _>(&value, serialized);

    let value = Value::Bool(true);
    assert_deserialize_with_value::<Tag, Value, _>(&value, serialized);
}

#[test]
fn test_u8_0() {
    type Tag = u8;
    let serialized = [3, 0];

    let value = 0u8;
    assert_serde_with_value::<Tag, u8, _>(&value, serialized);

    let value = Value::U8(0);
    assert_serde::<Tag, Value, _>(&value, serialized);
}

#[test]
fn test_u8_255() {
    type Tag = u8;
    let serialized = [3, 255];

    let value = 255u8;
    assert_serde_with_value::<Tag, u8, _>(&value, serialized);

    let value = Value::U8(255);
    assert_serde_with_value::<Tag, Value, _>(&value, serialized);
}

#[test]
fn test_i8_0() {
    type Tag = i8;
    let serialized = [4, 0];

    let value = 0i8;
    assert_serde_with_value::<Tag, i8, _>(&value, serialized);

    let value = Value::I8(0);
    assert_serde_with_value::<Tag, Value, _>(&value, serialized);
}

#[test]
fn test_i8_1() {
    type Tag = i8;
    let serialized = [4, 1];

    let value = 1i8;
    assert_serde_with_value::<Tag, i8, _>(&value, serialized);

    let value = Value::I8(1);
    assert_serde_with_value::<Tag, Value, _>(&value, serialized);
}

#[test]
fn test_i8_minus_1() {
    type Tag = i8;
    let serialized = [4, 255];

    let value = -1i8;
    assert_serde_with_value::<Tag, i8, _>(&value, serialized);

    let value = Value::I8(-1);
    assert_serde_with_value::<Tag, Value, _>(&value, serialized);
}

#[test]
fn test_i8_127() {
    type Tag = i8;
    let serialized = [4, 127];

    let value = 127i8;
    assert_serde_with_value::<Tag, i8, _>(&value, serialized);

    let value = Value::I8(127);
    assert_serde_with_value::<Tag, Value, _>(&value, serialized);
}

#[test]
fn test_i8_minus_128() {
    type Tag = i8;
    let serialized = [4, 128];

    let value = -128i8;
    assert_serde_with_value::<Tag, i8, _>(&value, serialized);

    let value = Value::I8(-128);
    assert_serde_with_value::<Tag, Value, _>(&value, serialized);
}

#[test]
fn test_u16_0() {
    type Tag = u16;
    let serialized = [5, 0];

    let value = 0u16;
    assert_serde_with_value::<Tag, u16, _>(&value, serialized);

    let value = Value::U16(0);
    assert_serde_with_value::<Tag, Value, _>(&value, serialized);
}

#[test]
fn test_u16_max() {
    type Tag = u16;
    let serialized = [5, 255, 255, 255];

    let value = u16::MAX;
    assert_serde_with_value::<Tag, u16, _>(&value, serialized);

    let value = Value::U16(u16::MAX);
    assert_serde_with_value::<Tag, Value, _>(&value, serialized);
}

#[test]
fn test_i16_0() {
    type Tag = i16;
    let serialized = [6, 0];

    let value = 0i16;
    assert_serde_with_value::<Tag, i16, _>(&value, serialized);

    let value = Value::I16(0);
    assert_serde_with_value::<Tag, Value, _>(&value, serialized);
}

#[test]
fn test_i16_1() {
    type Tag = i16;
    let serialized = [6, 2];

    let value = 1i16;
    assert_serde_with_value::<Tag, i16, _>(&value, serialized);

    let value = Value::I16(1);
    assert_serde_with_value::<Tag, Value, _>(&value, serialized);
}

#[test]
fn test_i16_minus_1() {
    type Tag = i16;
    let serialized = [6, 1];

    let value = -1i16;
    assert_serde_with_value::<Tag, i16, _>(&value, serialized);

    let value = Value::I16(-1);
    assert_serde_with_value::<Tag, Value, _>(&value, serialized);
}

#[test]
fn test_i16_max() {
    type Tag = i16;
    let serialized = [6, 255, 254, 255];

    let value = i16::MAX;
    assert_serde_with_value::<Tag, i16, _>(&value, serialized);

    let value = Value::I16(i16::MAX);
    assert_serde_with_value::<Tag, Value, _>(&value, serialized);
}

#[test]
fn test_i16_min() {
    type Tag = i16;
    let serialized = [6, 255, 255, 255];

    let value = i16::MIN;
    assert_serde_with_value::<Tag, i16, _>(&value, serialized);

    let value = Value::I16(i16::MIN);
    assert_serde_with_value::<Tag, Value, _>(&value, serialized);
}

#[test]
fn test_u32_0() {
    type Tag = u32;
    let serialized = [7, 0];

    let value = 0u32;
    assert_serde_with_value::<Tag, u32, _>(&value, serialized);

    let value = Value::U32(0);
    assert_serde_with_value::<Tag, Value, _>(&value, serialized);
}

#[test]
fn test_u32_max() {
    type Tag = u32;
    let serialized = [7, 255, 255, 255, 255, 255];

    let value = u32::MAX;
    assert_serde_with_value::<Tag, u32, _>(&value, serialized);

    let value = Value::U32(u32::MAX);
    assert_serde_with_value::<Tag, Value, _>(&value, serialized);
}

#[test]
fn test_i32_0() {
    type Tag = i32;
    let serialized = [8, 0];

    let value = 0i32;
    assert_serde_with_value::<Tag, i32, _>(&value, serialized);

    let value = Value::I32(0);
    assert_serde_with_value::<Tag, Value, _>(&value, serialized);
}

#[test]
fn test_i32_1() {
    type Tag = i32;
    let serialized = [8, 2];

    let value = 1i32;
    assert_serde_with_value::<Tag, i32, _>(&value, serialized);

    let value = Value::I32(1);
    assert_serde_with_value::<Tag, Value, _>(&value, serialized);
}

#[test]
fn test_i32_minus_1() {
    type Tag = i32;
    let serialized = [8, 1];

    let value = -1i32;
    assert_serde_with_value::<Tag, i32, _>(&value, serialized);

    let value = Value::I32(-1);
    assert_serde_with_value::<Tag, Value, _>(&value, serialized);
}

#[test]
fn test_i32_max() {
    type Tag = i32;
    let serialized = [8, 255, 254, 255, 255, 255];

    let value = i32::MAX;
    assert_serde_with_value::<Tag, i32, _>(&value, serialized);

    let value = Value::I32(i32::MAX);
    assert_serde_with_value::<Tag, Value, _>(&value, serialized);
}

#[test]
fn test_i32_min() {
    type Tag = i32;
    let serialized = [8, 255, 255, 255, 255, 255];

    let value = i32::MIN;
    assert_serde_with_value::<Tag, i32, _>(&value, serialized);

    let value = Value::I32(i32::MIN);
    assert_serde_with_value::<Tag, Value, _>(&value, serialized);
}

#[test]
fn test_u64_0() {
    type Tag = u64;
    let serialized = [9, 0];

    let value = 0u64;
    assert_serde_with_value::<Tag, u64, _>(&value, serialized);

    let value = Value::U64(0);
    assert_serde_with_value::<Tag, Value, _>(&value, serialized);
}

#[test]
fn test_u64_max() {
    type Tag = u64;
    let serialized = [9, 255, 255, 255, 255, 255, 255, 255, 255, 255];

    let value = u64::MAX;
    assert_serde_with_value::<Tag, u64, _>(&value, serialized);

    let value = Value::U64(u64::MAX);
    assert_serde_with_value::<Tag, Value, _>(&value, serialized);
}

#[test]
fn test_i64_0() {
    type Tag = i64;
    let serialized = [10, 0];

    let value = 0i64;
    assert_serde_with_value::<Tag, i64, _>(&value, serialized);

    let value = Value::I64(0);
    assert_serde_with_value::<Tag, Value, _>(&value, serialized);
}

#[test]
fn test_i64_1() {
    type Tag = i64;
    let serialized = [10, 2];

    let value = 1i64;
    assert_serde_with_value::<Tag, i64, _>(&value, serialized);

    let value = Value::I64(1);
    assert_serde_with_value::<Tag, Value, _>(&value, serialized);
}

#[test]
fn test_i64_minux_1() {
    type Tag = i64;
    let serialized = [10, 1];

    let value = -1i64;
    assert_serde_with_value::<Tag, i64, _>(&value, serialized);

    let value = Value::I64(-1);
    assert_serde_with_value::<Tag, Value, _>(&value, serialized);
}

#[test]
fn test_i64_max() {
    type Tag = i64;
    let serialized = [10, 255, 254, 255, 255, 255, 255, 255, 255, 255];

    let value = i64::MAX;
    assert_serde_with_value::<Tag, i64, _>(&value, serialized);

    let value = Value::I64(i64::MAX);
    assert_serde_with_value::<Tag, Value, _>(&value, serialized);
}

#[test]
fn test_i64_min() {
    type Tag = i64;
    let serialized = [10, 255, 255, 255, 255, 255, 255, 255, 255, 255];

    let value = i64::MIN;
    assert_serde_with_value::<Tag, i64, _>(&value, serialized);

    let value = Value::I64(i64::MIN);
    assert_serde_with_value::<Tag, Value, _>(&value, serialized);
}

#[test]
fn test_f32_0() {
    type Tag = f32;
    let serialized = [11, 0, 0, 0, 0];

    let value = 0f32;
    assert_serde_with_value::<Tag, f32, _>(&value, serialized);

    let value = Value::F32(0.0);
    assert_serde_with_value::<Tag, Value, _>(&value, serialized);
}

#[test]
fn test_f32_pi() {
    type Tag = f32;
    let serialized = [11, 219, 15, 73, 64];

    let value = f32::consts::PI;
    assert_serde_with_value::<Tag, f32, _>(&value, serialized);

    let value = Value::F32(f32::consts::PI);
    assert_serde_with_value::<Tag, Value, _>(&value, serialized);
}

#[test]
fn test_f64_0() {
    type Tag = f64;
    let serialized = [12, 0, 0, 0, 0, 0, 0, 0, 0];

    let value = 0f64;
    assert_serde_with_value::<Tag, f64, _>(&value, serialized);

    let value = Value::F64(0.0);
    assert_serde_with_value::<Tag, Value, _>(&value, serialized);
}

#[test]
fn test_f64_pi() {
    type Tag = f64;
    let serialized = [12, 24, 45, 68, 84, 251, 33, 9, 64];

    let value = f64::consts::PI;
    assert_serde_with_value::<Tag, f64, _>(&value, serialized);

    let value = Value::F64(f64::consts::PI);
    assert_serde_with_value::<Tag, Value, _>(&value, serialized);
}

#[test]
fn test_string_1() {
    type Tag = String;
    let serialized = [13, 4, b'a', b'b', b'c', b'd'];

    let value = "abcd".to_owned();
    assert_serde_with_value::<Tag, String, _>(&value, serialized);

    let value = Value::String("abcd".to_owned());
    assert_serde_with_value::<Tag, Value, _>(&value, serialized);

    let value = "abcd";
    assert_serialize_with_value::<Tag, &str, _>(&value, serialized);
}

#[test]
fn test_string_2() {
    type Tag = String;
    let serialized = [13, 6, 195, 164, 195, 182, 195, 188];

    let value = "äöü".to_owned();
    assert_serde_with_value::<Tag, String, _>(&value, serialized);

    let value = Value::String("äöü".to_owned());
    assert_serde_with_value::<Tag, Value, _>(&value, serialized);

    let value = "äöü";
    assert_serialize_with_value::<Tag, &str, _>(&value, serialized);
}

#[test]
fn test_string_empty() {
    type Tag = String;
    let serialized = [13, 0];

    let value = String::new();
    assert_serde_with_value::<Tag, String, _>(&value, serialized);

    let value = Value::String(String::new());
    assert_serde_with_value::<Tag, Value, _>(&value, serialized);

    let value = "";
    assert_serialize_with_value::<Tag, &str, _>(&value, serialized);
}

#[test]
fn test_uuid() {
    type Tag = Uuid;
    let uuid = uuid!("01234567-89ab-cdef-0246-8ace13579bdf");
    let serialized = [
        14, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x02, 0x46, 0x8a, 0xce, 0x13, 0x57,
        0x9b, 0xdf,
    ];

    let value = uuid;
    assert_serde_with_value::<Tag, Uuid, _>(&value, serialized);

    let value = Value::Uuid(uuid);
    assert_serde_with_value::<Tag, Value, _>(&value, serialized);

    let value = BusListenerCookie(uuid);
    assert_serde_with_value::<Tag, BusListenerCookie, _>(&value, serialized);

    let value = ChannelCookie(uuid);
    assert_serde_with_value::<Tag, ChannelCookie, _>(&value, serialized);

    let value = ObjectCookie(uuid);
    assert_serde_with_value::<Tag, ObjectCookie, _>(&value, serialized);

    let value = ObjectUuid(uuid);
    assert_serde_with_value::<Tag, ObjectUuid, _>(&value, serialized);

    let value = ServiceCookie(uuid);
    assert_serde_with_value::<Tag, ServiceCookie, _>(&value, serialized);

    let value = ServiceUuid(uuid);
    assert_serde_with_value::<Tag, ServiceUuid, _>(&value, serialized);

    let value = TypeId(uuid);
    assert_serde_with_value::<Tag, TypeId, _>(&value, serialized);
}

#[test]
fn test_object_id() {
    type Tag = ObjectId;
    let uuid = ObjectUuid(uuid!("81494c44-3bed-48e6-b078-1a93a1ae0e29"));
    let cookie = ObjectCookie(uuid!("a29885a9-0212-4940-964f-e7302131714b"));
    let id = ObjectId::new(uuid, cookie);
    let serialized = [
        15, 0x81, 0x49, 0x4c, 0x44, 0x3b, 0xed, 0x48, 0xe6, 0xb0, 0x78, 0x1a, 0x93, 0xa1, 0xae,
        0x0e, 0x29, 0xa2, 0x98, 0x85, 0xa9, 0x02, 0x12, 0x49, 0x40, 0x96, 0x4f, 0xe7, 0x30, 0x21,
        0x31, 0x71, 0x4b,
    ];

    let value = id;
    assert_serde_with_value::<Tag, ObjectId, _>(&value, serialized);

    let value = Value::ObjectId(id);
    assert_serde_with_value::<Tag, Value, _>(&value, serialized);
}

#[test]
fn test_service_id() {
    type Tag = ServiceId;
    let object_uuid = ObjectUuid(uuid!("81494c44-3bed-48e6-b078-1a93a1ae0e29"));
    let object_cookie = ObjectCookie(uuid!("a29885a9-0212-4940-964f-e7302131714b"));
    let object_id = ObjectId::new(object_uuid, object_cookie);
    let service_uuid = ServiceUuid(uuid!("042ed578-1e74-4365-94b0-3f76facfb8b4"));
    let service_cookie = ServiceCookie(uuid!("73e72e6b-12c3-49fc-9dfc-e4f0bf1917b1"));
    let id = ServiceId::new(object_id, service_uuid, service_cookie);
    let serialized = [
        16, 0x81, 0x49, 0x4c, 0x44, 0x3b, 0xed, 0x48, 0xe6, 0xb0, 0x78, 0x1a, 0x93, 0xa1, 0xae,
        0x0e, 0x29, 0xa2, 0x98, 0x85, 0xa9, 0x02, 0x12, 0x49, 0x40, 0x96, 0x4f, 0xe7, 0x30, 0x21,
        0x31, 0x71, 0x4b, 0x04, 0x2e, 0xd5, 0x78, 0x1e, 0x74, 0x43, 0x65, 0x94, 0xb0, 0x3f, 0x76,
        0xfa, 0xcf, 0xb8, 0xb4, 0x73, 0xe7, 0x2e, 0x6b, 0x12, 0xc3, 0x49, 0xfc, 0x9d, 0xfc, 0xe4,
        0xf0, 0xbf, 0x19, 0x17, 0xb1,
    ];

    let value = id;
    assert_serde_with_value::<Tag, ServiceId, _>(&value, serialized);

    let value = Value::ServiceId(id);
    assert_serde_with_value::<Tag, Value, _>(&value, serialized);
}

// TODO

#[test]
fn test_sender() {
    type Tag = Sender<()>;
    let uuid = uuid!("01234567-89ab-cdef-0246-8ace13579bdf");
    let serialized = [
        41, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x02, 0x46, 0x8a, 0xce, 0x13, 0x57,
        0x9b, 0xdf,
    ];

    let value = uuid;
    assert_serde::<Tag, Uuid, _>(&value, serialized);

    let value = Value::Sender(ChannelCookie(uuid));
    assert_serde_with_value::<Tag, Value, _>(&value, serialized);

    let value = ChannelCookie(uuid);
    assert_serde::<Tag, ChannelCookie, _>(&value, serialized);
}

#[test]
fn test_receiver() {
    type Tag = Receiver<()>;
    let uuid = uuid!("01234567-89ab-cdef-0246-8ace13579bdf");
    let serialized = [
        42, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x02, 0x46, 0x8a, 0xce, 0x13, 0x57,
        0x9b, 0xdf,
    ];

    let value = uuid;
    assert_serde::<Tag, Uuid, _>(&value, serialized);

    let value = Value::Receiver(ChannelCookie(uuid));
    assert_serde_with_value::<Tag, Value, _>(&value, serialized);

    let value = ChannelCookie(uuid);
    assert_serde::<Tag, ChannelCookie, _>(&value, serialized);
}

#[test]
fn test_vec() {
    type Tag = Vec<u8>;
    let serialized = [17, 2, 3, 7, 3, 8];

    let value = vec![7, 8];
    assert_serde_with_value::<Tag, Vec<u8>, _>(&value, serialized);

    let value = Value::Vec(vec![Value::U8(7), Value::U8(8)]);
    assert_serde_with_value::<Tag, Value, _>(&value, serialized);

    let value = VecDeque::from_iter([7, 8]);
    assert_serde_with_value::<Tag, VecDeque<u8>, _>(&value, serialized);

    let value = LinkedList::from_iter([7, 8]);
    assert_serde_with_value::<Tag, LinkedList<u8>, _>(&value, serialized);

    let value = &[7, 8][..];
    assert_serialize_with_value::<Tag, &[u8], _>(&value, serialized);

    let value = [7, 8];
    assert_serde_with_value::<Tag, [u8; 2], _>(&value, serialized);
}

#[test]
fn test_vec_value() {
    type Tag = Vec<Value>;
    let serialized = [17, 2, 0, 3, 4];

    let value = Value::Vec(vec![Value::None, Value::U8(4)]);
    assert_serde_with_value::<Tag, Value, _>(&value, serialized);
}
