use crate::tags::{self, Tag};
use crate::{
    BusListenerCookie, ByteSlice, Bytes, ChannelCookie, Deserialize, DeserializeError,
    Deserializer, ObjectCookie, ObjectId, ObjectUuid, ProtocolVersion, Serialize, SerializeError,
    SerializedValue, SerializedValueSlice, Serializer, ServiceCookie, ServiceId, ServiceUuid,
    TypeId, UnknownFields, UnknownVariant, Value, ValueConversionError,
};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, LinkedList, VecDeque};
use std::fmt::Debug;
use std::{f32, f64};
use uuid::{Uuid, uuid};

#[track_caller]
fn assert_serialize<'a, T, U, B>(value: &'a U, expected: B)
where
    T: Tag,
    U: Serialize<T> + Clone,
    &'a U: Serialize<T>,
    B: AsRef<[u8]>,
{
    let serialized_as_t = SerializedValue::serialize_as::<T>(value.clone()).unwrap();
    assert_eq!(serialized_as_t[..], *expected.as_ref());

    let serialized_as_t = SerializedValue::serialize_as::<T>(value).unwrap();
    assert_eq!(serialized_as_t[..], *expected.as_ref());
}

#[track_caller]
fn assert_deserialize_impl<T, U, B>(expected: &U, serialized: B)
where
    T: Tag,
    U: Deserialize<T> + PartialEq + Debug,
    B: AsRef<[u8]>,
{
    let serialized_value = SerializedValueSlice::new(&serialized);

    // Actual deserialization
    assert_eq!(*expected, serialized_value.deserialize_as().unwrap());

    // skip
    let mut buf = serialized.as_ref();
    Deserializer::new(&mut buf, 0).unwrap().skip().unwrap();
    assert_eq!(*buf, []);

    // len
    let mut buf = serialized.as_ref();
    let len = Deserializer::new(&mut buf, 0).unwrap().len().unwrap();
    assert_eq!(len, buf.len());
}

#[track_caller]
fn assert_deserialize<T, U, V1, V2>(expected: &U, v1: V1, v2: V2)
where
    T: Tag,
    U: Deserialize<T> + PartialEq + Debug,
    V1: AsRef<[u8]>,
    V2: AsRef<[u8]>,
{
    assert_deserialize_impl(expected, v1);
    assert_deserialize_impl(expected, v2);
}

#[track_caller]
fn assert_convert<V1, V2>(v1: V1, v2: V2)
where
    V1: AsRef<[u8]>,
    V2: AsRef<[u8]>,
{
    const V1: ProtocolVersion = ProtocolVersion::V1_14;
    const V2: ProtocolVersion = ProtocolVersion::V1_20;

    let v1 = SerializedValueSlice::new(&v1);
    let v2 = SerializedValueSlice::new(&v2);

    let v2_to_v1 = v2.convert(Some(V2), V1).unwrap();
    assert_eq!(v2_to_v1.as_ref(), v1);

    let v1_to_v1 = v1.convert(None, V1).unwrap();
    assert_eq!(v1_to_v1.as_ref(), v1);
}

#[track_caller]
fn assert_serde<'a, T, U, V1, V2>(value: &'a U, v1: V1, v2: V2)
where
    T: Tag,
    U: Serialize<T> + Deserialize<T> + Clone + PartialEq + Debug,
    &'a U: Serialize<T>,
    V1: AsRef<[u8]>,
    V2: AsRef<[u8]>,
{
    assert_serialize::<T, U, _>(value, &v2);
    assert_deserialize::<T, U, _, _>(value, &v1, &v2);
    assert_convert(&v1, &v2);
}

#[test]
fn test_unit() {
    type Tag = tags::Unit;
    let serialized = [0];

    let value = ();
    assert_serde::<Tag, (), _, _>(&value, serialized, serialized);

    let value = Value::None;
    assert_serde::<_, Value, _, _>(&value, serialized, serialized);
}

#[test]
fn test_option_none() {
    type Tag = tags::Option<tags::Unit>;
    let serialized = [0];

    let value = None;
    assert_serde::<Tag, Option<()>, _, _>(&value, serialized, serialized);

    let value = Value::None;
    assert_serde::<_, Value, _, _>(&value, serialized, serialized);

    let value = ();
    assert_serde::<Tag, (), _, _>(&value, serialized, serialized);
}

#[test]
fn test_option_some() {
    type Tag = tags::Option<tags::Unit>;
    let serialized = [1, 0];

    let value = Some(());
    assert_serde::<Tag, Option<()>, _, _>(&value, serialized, serialized);

    let value = Value::Some(Box::new(Value::None));
    assert_serde::<_, Value, _, _>(&value, serialized, serialized);
}

#[test]
fn test_bool_false() {
    type Tag = tags::Bool;
    let serialized = [2, 0];

    let value = false;
    assert_serde::<Tag, bool, _, _>(&value, serialized, serialized);

    let value = Value::Bool(false);
    assert_serde::<_, Value, _, _>(&value, serialized, serialized);
}

#[test]
fn test_bool_true() {
    type Tag = tags::Bool;
    let serialized = [2, 1];

    let value = true;
    assert_serde::<Tag, bool, _, _>(&value, serialized, serialized);

    let value = Value::Bool(true);
    assert_serde::<_, Value, _, _>(&value, serialized, serialized);
}

#[test]
fn test_bool_non_zero() {
    type Tag = tags::Bool;
    let serialized = [2, 2];

    let value = true;
    assert_deserialize::<Tag, bool, _, _>(&value, serialized, serialized);

    let value = Value::Bool(true);
    assert_deserialize::<_, Value, _, _>(&value, serialized, serialized);
}

#[test]
fn test_u8_0() {
    type Tag = tags::U8;
    let serialized = [3, 0];

    let value = 0u8;
    assert_serde::<Tag, u8, _, _>(&value, serialized, serialized);

    let value = Value::U8(0);
    assert_serde::<_, Value, _, _>(&value, serialized, serialized);
}

#[test]
fn test_u8_255() {
    type Tag = tags::U8;
    let serialized = [3, 255];

    let value = 255u8;
    assert_serde::<Tag, u8, _, _>(&value, serialized, serialized);

    let value = Value::U8(255);
    assert_serde::<_, Value, _, _>(&value, serialized, serialized);
}

#[test]
fn test_i8_0() {
    type Tag = tags::I8;
    let serialized = [4, 0];

    let value = 0i8;
    assert_serde::<Tag, i8, _, _>(&value, serialized, serialized);

    let value = Value::I8(0);
    assert_serde::<_, Value, _, _>(&value, serialized, serialized);
}

#[test]
fn test_i8_1() {
    type Tag = tags::I8;
    let serialized = [4, 1];

    let value = 1i8;
    assert_serde::<Tag, i8, _, _>(&value, serialized, serialized);

    let value = Value::I8(1);
    assert_serde::<_, Value, _, _>(&value, serialized, serialized);
}

#[test]
fn test_i8_minus_1() {
    type Tag = tags::I8;
    let serialized = [4, 255];

    let value = -1i8;
    assert_serde::<Tag, i8, _, _>(&value, serialized, serialized);

    let value = Value::I8(-1);
    assert_serde::<_, Value, _, _>(&value, serialized, serialized);
}

#[test]
fn test_i8_127() {
    type Tag = tags::I8;
    let serialized = [4, 127];

    let value = 127i8;
    assert_serde::<Tag, i8, _, _>(&value, serialized, serialized);

    let value = Value::I8(127);
    assert_serde::<_, Value, _, _>(&value, serialized, serialized);
}

#[test]
fn test_i8_minus_128() {
    type Tag = tags::I8;
    let serialized = [4, 128];

    let value = -128i8;
    assert_serde::<Tag, i8, _, _>(&value, serialized, serialized);

    let value = Value::I8(-128);
    assert_serde::<_, Value, _, _>(&value, serialized, serialized);
}

#[test]
fn test_u16_0() {
    type Tag = tags::U16;
    let serialized = [5, 0];

    let value = 0u16;
    assert_serde::<Tag, u16, _, _>(&value, serialized, serialized);

    let value = Value::U16(0);
    assert_serde::<_, Value, _, _>(&value, serialized, serialized);
}

#[test]
fn test_u16_max() {
    type Tag = tags::U16;
    let serialized = [5, 255, 255, 255];

    let value = u16::MAX;
    assert_serde::<Tag, u16, _, _>(&value, serialized, serialized);

    let value = Value::U16(u16::MAX);
    assert_serde::<_, Value, _, _>(&value, serialized, serialized);
}

#[test]
fn test_i16_0() {
    type Tag = tags::I16;
    let serialized = [6, 0];

    let value = 0i16;
    assert_serde::<Tag, i16, _, _>(&value, serialized, serialized);

    let value = Value::I16(0);
    assert_serde::<_, Value, _, _>(&value, serialized, serialized);
}

#[test]
fn test_i16_1() {
    type Tag = tags::I16;
    let serialized = [6, 2];

    let value = 1i16;
    assert_serde::<Tag, i16, _, _>(&value, serialized, serialized);

    let value = Value::I16(1);
    assert_serde::<_, Value, _, _>(&value, serialized, serialized);
}

#[test]
fn test_i16_minus_1() {
    type Tag = tags::I16;
    let serialized = [6, 1];

    let value = -1i16;
    assert_serde::<Tag, i16, _, _>(&value, serialized, serialized);

    let value = Value::I16(-1);
    assert_serde::<_, Value, _, _>(&value, serialized, serialized);
}

#[test]
fn test_i16_max() {
    type Tag = tags::I16;
    let serialized = [6, 255, 254, 255];

    let value = i16::MAX;
    assert_serde::<Tag, i16, _, _>(&value, serialized, serialized);

    let value = Value::I16(i16::MAX);
    assert_serde::<_, Value, _, _>(&value, serialized, serialized);
}

#[test]
fn test_i16_min() {
    type Tag = tags::I16;
    let serialized = [6, 255, 255, 255];

    let value = i16::MIN;
    assert_serde::<Tag, i16, _, _>(&value, serialized, serialized);

    let value = Value::I16(i16::MIN);
    assert_serde::<_, Value, _, _>(&value, serialized, serialized);
}

#[test]
fn test_u32_0() {
    type Tag = tags::U32;
    let serialized = [7, 0];

    let value = 0u32;
    assert_serde::<Tag, u32, _, _>(&value, serialized, serialized);

    let value = Value::U32(0);
    assert_serde::<_, Value, _, _>(&value, serialized, serialized);
}

#[test]
fn test_u32_max() {
    type Tag = tags::U32;
    let serialized = [7, 255, 255, 255, 255, 255];

    let value = u32::MAX;
    assert_serde::<Tag, u32, _, _>(&value, serialized, serialized);

    let value = Value::U32(u32::MAX);
    assert_serde::<_, Value, _, _>(&value, serialized, serialized);
}

#[test]
fn test_i32_0() {
    type Tag = tags::I32;
    let serialized = [8, 0];

    let value = 0i32;
    assert_serde::<Tag, i32, _, _>(&value, serialized, serialized);

    let value = Value::I32(0);
    assert_serde::<_, Value, _, _>(&value, serialized, serialized);
}

#[test]
fn test_i32_1() {
    type Tag = tags::I32;
    let serialized = [8, 2];

    let value = 1i32;
    assert_serde::<Tag, i32, _, _>(&value, serialized, serialized);

    let value = Value::I32(1);
    assert_serde::<_, Value, _, _>(&value, serialized, serialized);
}

#[test]
fn test_i32_minus_1() {
    type Tag = tags::I32;
    let serialized = [8, 1];

    let value = -1i32;
    assert_serde::<Tag, i32, _, _>(&value, serialized, serialized);

    let value = Value::I32(-1);
    assert_serde::<_, Value, _, _>(&value, serialized, serialized);
}

#[test]
fn test_i32_max() {
    type Tag = tags::I32;
    let serialized = [8, 255, 254, 255, 255, 255];

    let value = i32::MAX;
    assert_serde::<Tag, i32, _, _>(&value, serialized, serialized);

    let value = Value::I32(i32::MAX);
    assert_serde::<_, Value, _, _>(&value, serialized, serialized);
}

#[test]
fn test_i32_min() {
    type Tag = tags::I32;
    let serialized = [8, 255, 255, 255, 255, 255];

    let value = i32::MIN;
    assert_serde::<Tag, i32, _, _>(&value, serialized, serialized);

    let value = Value::I32(i32::MIN);
    assert_serde::<_, Value, _, _>(&value, serialized, serialized);
}

#[test]
fn test_u64_0() {
    type Tag = tags::U64;
    let serialized = [9, 0];

    let value = 0u64;
    assert_serde::<Tag, u64, _, _>(&value, serialized, serialized);

    let value = Value::U64(0);
    assert_serde::<_, Value, _, _>(&value, serialized, serialized);
}

#[test]
fn test_u64_max() {
    type Tag = tags::U64;
    let serialized = [9, 255, 255, 255, 255, 255, 255, 255, 255, 255];

    let value = u64::MAX;
    assert_serde::<Tag, u64, _, _>(&value, serialized, serialized);

    let value = Value::U64(u64::MAX);
    assert_serde::<_, Value, _, _>(&value, serialized, serialized);
}

#[test]
fn test_i64_0() {
    type Tag = tags::I64;
    let serialized = [10, 0];

    let value = 0i64;
    assert_serde::<Tag, i64, _, _>(&value, serialized, serialized);

    let value = Value::I64(0);
    assert_serde::<_, Value, _, _>(&value, serialized, serialized);
}

#[test]
fn test_i64_1() {
    type Tag = tags::I64;
    let serialized = [10, 2];

    let value = 1i64;
    assert_serde::<Tag, i64, _, _>(&value, serialized, serialized);

    let value = Value::I64(1);
    assert_serde::<_, Value, _, _>(&value, serialized, serialized);
}

#[test]
fn test_i64_minux_1() {
    type Tag = tags::I64;
    let serialized = [10, 1];

    let value = -1i64;
    assert_serde::<Tag, i64, _, _>(&value, serialized, serialized);

    let value = Value::I64(-1);
    assert_serde::<_, Value, _, _>(&value, serialized, serialized);
}

#[test]
fn test_i64_max() {
    type Tag = tags::I64;
    let serialized = [10, 255, 254, 255, 255, 255, 255, 255, 255, 255];

    let value = i64::MAX;
    assert_serde::<Tag, i64, _, _>(&value, serialized, serialized);

    let value = Value::I64(i64::MAX);
    assert_serde::<_, Value, _, _>(&value, serialized, serialized);
}

#[test]
fn test_i64_min() {
    type Tag = tags::I64;
    let serialized = [10, 255, 255, 255, 255, 255, 255, 255, 255, 255];

    let value = i64::MIN;
    assert_serde::<Tag, i64, _, _>(&value, serialized, serialized);

    let value = Value::I64(i64::MIN);
    assert_serde::<_, Value, _, _>(&value, serialized, serialized);
}

#[test]
fn test_f32_0() {
    type Tag = tags::F32;
    let serialized = [11, 0, 0, 0, 0];

    let value = 0f32;
    assert_serde::<Tag, f32, _, _>(&value, serialized, serialized);

    let value = Value::F32(0.0);
    assert_serde::<_, Value, _, _>(&value, serialized, serialized);
}

#[test]
fn test_f32_pi() {
    type Tag = tags::F32;
    let serialized = [11, 219, 15, 73, 64];

    let value = f32::consts::PI;
    assert_serde::<Tag, f32, _, _>(&value, serialized, serialized);

    let value = Value::F32(f32::consts::PI);
    assert_serde::<_, Value, _, _>(&value, serialized, serialized);
}

#[test]
fn test_f64_0() {
    type Tag = tags::F64;
    let serialized = [12, 0, 0, 0, 0, 0, 0, 0, 0];

    let value = 0f64;
    assert_serde::<Tag, f64, _, _>(&value, serialized, serialized);

    let value = Value::F64(0.0);
    assert_serde::<_, Value, _, _>(&value, serialized, serialized);
}

#[test]
fn test_f64_pi() {
    type Tag = tags::F64;
    let serialized = [12, 24, 45, 68, 84, 251, 33, 9, 64];

    let value = f64::consts::PI;
    assert_serde::<Tag, f64, _, _>(&value, serialized, serialized);

    let value = Value::F64(f64::consts::PI);
    assert_serde::<_, Value, _, _>(&value, serialized, serialized);
}

#[test]
fn test_string_1() {
    type Tag = tags::String;
    let serialized = [13, 4, b'a', b'b', b'c', b'd'];

    let value = "abcd".to_owned();
    assert_serde::<Tag, String, _, _>(&value, serialized, serialized);

    let value = Value::String("abcd".to_owned());
    assert_serde::<_, Value, _, _>(&value, serialized, serialized);

    let value = "abcd";
    assert_serialize::<Tag, &str, _>(&value, serialized);
}

#[test]
fn test_string_2() {
    type Tag = tags::String;
    let serialized = [13, 6, 195, 164, 195, 182, 195, 188];

    let value = "äöü".to_owned();
    assert_serde::<Tag, String, _, _>(&value, serialized, serialized);

    let value = Value::String("äöü".to_owned());
    assert_serde::<_, Value, _, _>(&value, serialized, serialized);

    let value = "äöü";
    assert_serialize::<Tag, &str, _>(&value, serialized);
}

#[test]
fn test_string_empty() {
    type Tag = tags::String;
    let serialized = [13, 0];

    let value = String::new();
    assert_serde::<Tag, String, _, _>(&value, serialized, serialized);

    let value = Value::String(String::new());
    assert_serde::<_, Value, _, _>(&value, serialized, serialized);

    let value = "";
    assert_serialize::<Tag, &str, _>(&value, serialized);
}

#[test]
fn test_uuid() {
    type Tag = tags::Uuid;
    let uuid = uuid!("01234567-89ab-cdef-0246-8ace13579bdf");
    let serialized = [
        14, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x02, 0x46, 0x8a, 0xce, 0x13, 0x57,
        0x9b, 0xdf,
    ];

    let value = uuid;
    assert_serde::<Tag, Uuid, _, _>(&value, serialized, serialized);

    let value = Value::Uuid(uuid);
    assert_serde::<_, Value, _, _>(&value, serialized, serialized);

    let value = BusListenerCookie(uuid);
    assert_serde::<Tag, BusListenerCookie, _, _>(&value, serialized, serialized);

    let value = ChannelCookie(uuid);
    assert_serde::<Tag, ChannelCookie, _, _>(&value, serialized, serialized);

    let value = ObjectCookie(uuid);
    assert_serde::<Tag, ObjectCookie, _, _>(&value, serialized, serialized);

    let value = ObjectUuid(uuid);
    assert_serde::<Tag, ObjectUuid, _, _>(&value, serialized, serialized);

    let value = ServiceCookie(uuid);
    assert_serde::<Tag, ServiceCookie, _, _>(&value, serialized, serialized);

    let value = ServiceUuid(uuid);
    assert_serde::<Tag, ServiceUuid, _, _>(&value, serialized, serialized);

    let value = TypeId(uuid);
    assert_serde::<Tag, TypeId, _, _>(&value, serialized, serialized);
}

#[test]
fn test_object_id() {
    type Tag = tags::ObjectId;
    let uuid = ObjectUuid(uuid!("81494c44-3bed-48e6-b078-1a93a1ae0e29"));
    let cookie = ObjectCookie(uuid!("a29885a9-0212-4940-964f-e7302131714b"));
    let id = ObjectId::new(uuid, cookie);
    let serialized = [
        15, 0x81, 0x49, 0x4c, 0x44, 0x3b, 0xed, 0x48, 0xe6, 0xb0, 0x78, 0x1a, 0x93, 0xa1, 0xae,
        0x0e, 0x29, 0xa2, 0x98, 0x85, 0xa9, 0x02, 0x12, 0x49, 0x40, 0x96, 0x4f, 0xe7, 0x30, 0x21,
        0x31, 0x71, 0x4b,
    ];

    let value = id;
    assert_serde::<Tag, ObjectId, _, _>(&value, serialized, serialized);

    let value = Value::ObjectId(id);
    assert_serde::<_, Value, _, _>(&value, serialized, serialized);
}

#[test]
fn test_service_id() {
    type Tag = tags::ServiceId;
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
    assert_serde::<Tag, ServiceId, _, _>(&value, serialized, serialized);

    let value = Value::ServiceId(id);
    assert_serde::<_, Value, _, _>(&value, serialized, serialized);
}

#[test]
fn test_vec() {
    type Tag = tags::Vec<tags::U8>;
    let v1 = [17, 2, 3, 7, 3, 8];
    let v2 = [43, 1, 3, 7, 1, 3, 8, 0];

    let value = vec![7, 8];
    assert_serde::<Tag, Vec<u8>, _, _>(&value, v1, v2);

    let value = Value::Vec(vec![Value::U8(7), Value::U8(8)]);
    assert_serde::<_, Value, _, _>(&value, v1, v2);

    let value = VecDeque::from_iter([7, 8]);
    assert_serde::<Tag, VecDeque<u8>, _, _>(&value, v1, v2);

    let value = LinkedList::from_iter([7, 8]);
    assert_serde::<Tag, LinkedList<u8>, _, _>(&value, v1, v2);

    let value = &[7, 8][..];
    assert_serialize::<Tag, &[u8], _>(&value, v2);

    let value = [7, 8];
    assert_serde::<Tag, [u8; 2], _, _>(&value, v1, v2);

    let value = Bytes::new([7, 8]);
    assert_serde::<Tag, Bytes, _, _>(&value, v1, v2);

    let value = ByteSlice::new(&[7, 8]);
    assert_serialize::<Tag, &ByteSlice, _>(&value, v2);

    let value = bytes::Bytes::from(&[7, 8][..]);
    assert_serde::<Tag, bytes::Bytes, _, _>(&value, v1, v2);

    let value = bytes::BytesMut::from(&[7, 8][..]);
    assert_serde::<Tag, bytes::BytesMut, _, _>(&value, v1, v2);
}

#[test]
fn test_vec_value() {
    let v1 = [17, 2, 0, 3, 4];
    let v2 = [43, 1, 0, 1, 3, 4, 0];

    let value = Value::Vec(vec![Value::None, Value::U8(4)]);
    assert_serde::<_, Value, _, _>(&value, v1, v2);
}

#[test]
fn test_vec_empty() {
    type Tag = tags::Vec<tags::Value>;
    let v1 = [17, 0];
    let v2 = [43, 0];

    let value = ();
    assert_serde::<Tag, (), _, _>(&value, v1, v2);
}

#[test]
fn test_vec_error() {
    // This test specifically targets some unsafe clean-up code in impls/vec.rs. The serialized
    // value contains only one string, but then 2 strings are attempted to deserialize from it. This
    // fails an the first string, that is already deserialized, must be dropped.
    let serialized_value = SerializedValueSlice::new(&[43, 1, 13, 0, 0]);
    let res = serialized_value.deserialize::<[String; 2]>();
    assert_eq!(res, Err(DeserializeError::NoMoreElements));
}

#[test]
fn test_bytes() {
    type Tag = tags::Bytes;
    let bytes = [1, 2, 3];
    let v1 = [18, 3, 1, 2, 3];
    let v2 = [44, 3, 1, 2, 3, 0];

    let value = Bytes::new(bytes);
    assert_serde::<Tag, Bytes, _, _>(&value, v1, v2);

    let value = ByteSlice::new(&bytes);
    assert_serialize::<Tag, &ByteSlice, _>(&value, v2);

    let value = Value::Bytes(Bytes::new(bytes));
    assert_serde::<_, Value, _, _>(&value, v1, v2);

    let value = Vec::from(bytes);
    assert_serde::<Tag, Vec<u8>, _, _>(&value, v1, v2);

    let value = VecDeque::from_iter(bytes);
    assert_serde::<Tag, VecDeque<u8>, _, _>(&value, v1, v2);

    let value = &bytes[..];
    assert_serialize::<Tag, &[u8], _>(&value, v2);

    let value = bytes;
    assert_serde::<Tag, [u8; 3], _, _>(&value, v1, v2);

    let value = bytes::Bytes::from_iter(bytes);
    assert_serde::<Tag, bytes::Bytes, _, _>(&value, v1, v2);

    let value = bytes::BytesMut::from_iter(bytes);
    assert_serde::<Tag, bytes::BytesMut, _, _>(&value, v1, v2);
}

#[test]
fn test_bytes_linked_list() {
    type Tag = tags::Bytes;
    let bytes = [1, 2, 3];
    let serialized = [18, 3, 1, 2, 3];

    let value = LinkedList::from_iter(bytes);
    assert_serde::<Tag, LinkedList<u8>, _, _>(&value, serialized, serialized);
}

#[test]
fn test_bytes_segmented() {
    type Tag = tags::Bytes;
    let bytes = [1, 2, 3];
    let v1 = [18, 3, 1, 2, 3];
    let v2 = [44, 1, 1, 1, 2, 1, 3, 0];

    let value = Bytes::new(bytes);
    assert_deserialize::<Tag, Bytes, _, _>(&value, v1, v2);

    let value = Value::Bytes(Bytes::new(bytes));
    assert_deserialize::<_, Value, _, _>(&value, v1, v2);

    let value = Vec::from(bytes);
    assert_deserialize::<Tag, Vec<u8>, _, _>(&value, v1, v2);

    let value = VecDeque::from_iter(bytes);
    assert_deserialize::<Tag, VecDeque<u8>, _, _>(&value, v1, v2);

    let value = LinkedList::from_iter(bytes);
    assert_deserialize::<Tag, LinkedList<u8>, _, _>(&value, v1, v2);

    let value = bytes;
    assert_deserialize::<Tag, [u8; 3], _, _>(&value, v1, v2);

    let value = bytes::Bytes::from_iter(bytes);
    assert_deserialize::<Tag, bytes::Bytes, _, _>(&value, v1, v2);

    let value = bytes::BytesMut::from_iter(bytes);
    assert_deserialize::<Tag, bytes::BytesMut, _, _>(&value, v1, v2);
}

#[test]
fn test_bytes_empty() {
    type Tag = tags::Bytes;
    let v1 = [18, 0];
    let v2 = [44, 0];

    let value = ();
    assert_serde::<Tag, (), _, _>(&value, v1, v2);
}

#[test]
fn test_u8_map() {
    type Tag = tags::Map<tags::U8, tags::U8>;
    let v1_1 = [19, 2, 0, 3, 1, 2, 3, 3];
    let v1_2 = [19, 2, 2, 3, 3, 0, 3, 1];
    let v2_1 = [45, 1, 0, 3, 1, 1, 2, 3, 3, 0];
    let v2_2 = [45, 1, 2, 3, 3, 1, 0, 3, 1, 0];

    let value = HashMap::from_iter([(0, 1), (2, 3)]);
    if value.keys().next() == Some(&0) {
        assert_serde::<Tag, HashMap<u8, u8>, _, _>(&value, v1_1, v2_1);
    } else {
        assert_serde::<Tag, HashMap<u8, u8>, _, _>(&value, v1_2, v2_2);
    }

    let value = HashMap::from_iter([(0, Value::U8(1)), (2, Value::U8(3))]);
    if value.keys().next() == Some(&0) {
        let value = Value::U8Map(value);
        assert_serde::<_, Value, _, _>(&value, v1_1, v2_1);
    } else {
        let value = Value::U8Map(value);
        assert_serde::<_, Value, _, _>(&value, v1_2, v2_2);
    }

    let value = BTreeMap::from_iter([(0, 1), (2, 3)]);
    assert_serde::<Tag, BTreeMap<u8, u8>, _, _>(&value, v1_1, v2_1);
}

#[test]
fn test_u8_map_empty() {
    type Tag = tags::Map<tags::U8, tags::U8>;
    let v1 = [19, 0];
    let v2 = [45, 0];

    let value = ();
    assert_serde::<Tag, (), _, _>(&value, v1, v2);
}

#[test]
fn test_i8_map() {
    type Tag = tags::Map<tags::I8, tags::U8>;
    let v1 = [20, 1, 2, 3, 4];
    let v2 = [46, 1, 2, 3, 4, 0];

    let value = HashMap::from_iter([(2, 4)]);
    assert_serde::<Tag, HashMap<i8, u8>, _, _>(&value, v1, v2);

    let value = Value::I8Map(HashMap::from_iter([(2, Value::U8(4))]));
    assert_serde::<_, Value, _, _>(&value, v1, v2);

    let value = BTreeMap::from_iter([(2, 4)]);
    assert_serde::<Tag, BTreeMap<i8, u8>, _, _>(&value, v1, v2);
}

#[test]
fn test_u16_map() {
    type Tag = tags::Map<tags::U16, tags::U8>;
    let v1 = [21, 1, 2, 3, 4];
    let v2 = [47, 1, 2, 3, 4, 0];

    let value = HashMap::from_iter([(2, 4)]);
    assert_serde::<Tag, HashMap<u16, u8>, _, _>(&value, v1, v2);

    let value = Value::U16Map(HashMap::from_iter([(2, Value::U8(4))]));
    assert_serde::<_, Value, _, _>(&value, v1, v2);

    let value = BTreeMap::from_iter([(2, 4)]);
    assert_serde::<Tag, BTreeMap<u16, u8>, _, _>(&value, v1, v2);
}

#[test]
fn test_i16_map() {
    type Tag = tags::Map<tags::I16, tags::U8>;
    let v1 = [22, 1, 4, 3, 4];
    let v2 = [48, 1, 4, 3, 4, 0];

    let value = HashMap::from_iter([(2, 4)]);
    assert_serde::<Tag, HashMap<i16, u8>, _, _>(&value, v1, v2);

    let value = Value::I16Map(HashMap::from_iter([(2, Value::U8(4))]));
    assert_serde::<_, Value, _, _>(&value, v1, v2);

    let value = BTreeMap::from_iter([(2, 4)]);
    assert_serde::<Tag, BTreeMap<i16, u8>, _, _>(&value, v1, v2);
}

#[test]
fn test_u32_map() {
    type Tag = tags::Map<tags::U32, tags::U8>;
    let v1 = [23, 1, 2, 3, 4];
    let v2 = [49, 1, 2, 3, 4, 0];

    let value = HashMap::from_iter([(2, 4)]);
    assert_serde::<Tag, HashMap<u32, u8>, _, _>(&value, v1, v2);

    let value = Value::U32Map(HashMap::from_iter([(2, Value::U8(4))]));
    assert_serde::<_, Value, _, _>(&value, v1, v2);

    let value = BTreeMap::from_iter([(2, 4)]);
    assert_serde::<Tag, BTreeMap<u32, u8>, _, _>(&value, v1, v2);
}

#[test]
fn test_i32_map() {
    type Tag = tags::Map<tags::I32, tags::U8>;
    let v1 = [24, 1, 4, 3, 4];
    let v2 = [50, 1, 4, 3, 4, 0];

    let value = HashMap::from_iter([(2, 4)]);
    assert_serde::<Tag, HashMap<i32, u8>, _, _>(&value, v1, v2);

    let value = Value::I32Map(HashMap::from_iter([(2, Value::U8(4))]));
    assert_serde::<_, Value, _, _>(&value, v1, v2);

    let value = BTreeMap::from_iter([(2, 4)]);
    assert_serde::<Tag, BTreeMap<i32, u8>, _, _>(&value, v1, v2);
}

#[test]
fn test_u64_map() {
    type Tag = tags::Map<tags::U64, tags::U8>;
    let v1 = [25, 1, 2, 3, 4];
    let v2 = [51, 1, 2, 3, 4, 0];

    let value = HashMap::from_iter([(2, 4)]);
    assert_serde::<Tag, HashMap<u64, u8>, _, _>(&value, v1, v2);

    let value = Value::U64Map(HashMap::from_iter([(2, Value::U8(4))]));
    assert_serde::<_, Value, _, _>(&value, v1, v2);

    let value = BTreeMap::from_iter([(2, 4)]);
    assert_serde::<Tag, BTreeMap<u64, u8>, _, _>(&value, v1, v2);
}

#[test]
fn test_i64_map() {
    type Tag = tags::Map<tags::I64, tags::U8>;
    let v1 = [26, 1, 4, 3, 4];
    let v2 = [52, 1, 4, 3, 4, 0];

    let value = HashMap::from_iter([(2, 4)]);
    assert_serde::<Tag, HashMap<i64, u8>, _, _>(&value, v1, v2);

    let value = Value::I64Map(HashMap::from_iter([(2, Value::U8(4))]));
    assert_serde::<_, Value, _, _>(&value, v1, v2);

    let value = BTreeMap::from_iter([(2, 4)]);
    assert_serde::<Tag, BTreeMap<i64, u8>, _, _>(&value, v1, v2);
}

#[test]
fn test_string_map() {
    type Tag = tags::Map<tags::String, tags::U16>;
    let v1 = [27, 1, 2, b'3', b'4', 5, 6];
    let v2 = [53, 1, 2, b'3', b'4', 5, 6, 0];

    let value = HashMap::from_iter([("34".to_owned(), 6)]);
    assert_serde::<Tag, HashMap<String, u16>, _, _>(&value, v1, v2);

    let value = HashMap::from_iter([("34", 6)]);
    assert_serialize::<Tag, HashMap<&str, u16>, _>(&value, v2);

    let value = Value::StringMap(HashMap::from_iter([("34".to_owned(), Value::U16(6))]));
    assert_serde::<_, Value, _, _>(&value, v1, v2);

    let value = BTreeMap::from_iter([("34".to_owned(), 6)]);
    assert_serde::<Tag, BTreeMap<String, u16>, _, _>(&value, v1, v2);

    let value = BTreeMap::from_iter([("34", 6)]);
    assert_serialize::<Tag, BTreeMap<&str, u16>, _>(&value, v2);
}

#[test]
fn test_uuid_map() {
    type Tag = tags::Map<tags::Uuid, tags::U8>;
    let uuid = uuid!("81494c44-3bed-48e6-b078-1a93a1ae0e29");
    let v1 = [
        28, 1, 0x81, 0x49, 0x4c, 0x44, 0x3b, 0xed, 0x48, 0xe6, 0xb0, 0x78, 0x1a, 0x93, 0xa1, 0xae,
        0x0e, 0x29, 3, 0,
    ];
    let v2 = [
        54, 1, 0x81, 0x49, 0x4c, 0x44, 0x3b, 0xed, 0x48, 0xe6, 0xb0, 0x78, 0x1a, 0x93, 0xa1, 0xae,
        0x0e, 0x29, 3, 0, 0,
    ];

    let value = HashMap::from_iter([(uuid, 0)]);
    assert_serde::<Tag, HashMap<Uuid, u8>, _, _>(&value, v1, v2);

    let value = Value::UuidMap(HashMap::from_iter([(uuid, Value::U8(0))]));
    assert_serde::<_, Value, _, _>(&value, v1, v2);

    let value = BTreeMap::from_iter([(uuid, 0)]);
    assert_serde::<Tag, BTreeMap<Uuid, u8>, _, _>(&value, v1, v2);
}

#[test]
fn test_u8_set() {
    type Tag = tags::Set<tags::U8>;
    let v1_1 = [29, 2, 3, 4];
    let v1_2 = [29, 2, 4, 3];
    let v2_1 = [55, 1, 3, 1, 4, 0];
    let v2_2 = [55, 1, 4, 1, 3, 0];

    let value = HashSet::from_iter([3, 4]);
    if value.iter().next() == Some(&3) {
        assert_serde::<Tag, HashSet<u8>, _, _>(&value, v1_1, v2_1);
    } else {
        assert_serde::<Tag, HashSet<u8>, _, _>(&value, v1_2, v2_2);
    }

    let value = HashSet::from_iter([3, 4]);
    if value.iter().next() == Some(&3) {
        let value = Value::U8Set(value);
        assert_serde::<_, Value, _, _>(&value, v1_1, v2_1);
    } else {
        let value = Value::U8Set(value);
        assert_serde::<_, Value, _, _>(&value, v1_2, v2_2);
    }

    let value = BTreeSet::from_iter([3, 4]);
    assert_serde::<Tag, BTreeSet<u8>, _, _>(&value, v1_1, v2_1);
}

#[test]
fn test_u8_set_empty() {
    type Tag = tags::Set<tags::U8>;
    let v1 = [29, 0];
    let v2 = [55, 0];

    let value = ();
    assert_serde::<Tag, (), _, _>(&value, v1, v2);
}

#[test]
fn test_i8_set() {
    type Tag = tags::Set<tags::I8>;
    let v1 = [30, 1, 2];
    let v2 = [56, 1, 2, 0];

    let value = HashSet::from_iter([2]);
    assert_serde::<Tag, HashSet<i8>, _, _>(&value, v1, v2);

    let value = Value::I8Set(HashSet::from_iter([2]));
    assert_serde::<_, Value, _, _>(&value, v1, v2);

    let value = BTreeSet::from_iter([2]);
    assert_serde::<Tag, BTreeSet<i8>, _, _>(&value, v1, v2);
}

#[test]
fn test_u16_set() {
    type Tag = tags::Set<tags::U16>;
    let v1 = [31, 1, 2];
    let v2 = [57, 1, 2, 0];

    let value = HashSet::from_iter([2]);
    assert_serde::<Tag, HashSet<u16>, _, _>(&value, v1, v2);

    let value = Value::U16Set(HashSet::from_iter([2]));
    assert_serde::<_, Value, _, _>(&value, v1, v2);

    let value = BTreeSet::from_iter([2]);
    assert_serde::<Tag, BTreeSet<u16>, _, _>(&value, v1, v2);
}

#[test]
fn test_i16_set() {
    type Tag = tags::Set<tags::I16>;
    let v1 = [32, 1, 2];
    let v2 = [58, 1, 2, 0];

    let value = HashSet::from_iter([1]);
    assert_serde::<Tag, HashSet<i16>, _, _>(&value, v1, v2);

    let value = Value::I16Set(HashSet::from_iter([1]));
    assert_serde::<_, Value, _, _>(&value, v1, v2);

    let value = BTreeSet::from_iter([1]);
    assert_serde::<Tag, BTreeSet<i16>, _, _>(&value, v1, v2);
}

#[test]
fn test_u32_set() {
    type Tag = tags::Set<tags::U32>;
    let v1 = [33, 1, 2];
    let v2 = [59, 1, 2, 0];

    let value = HashSet::from_iter([2]);
    assert_serde::<Tag, HashSet<u32>, _, _>(&value, v1, v2);

    let value = Value::U32Set(HashSet::from_iter([2]));
    assert_serde::<_, Value, _, _>(&value, v1, v2);

    let value = BTreeSet::from_iter([2]);
    assert_serde::<Tag, BTreeSet<u32>, _, _>(&value, v1, v2);
}

#[test]
fn test_i32_set() {
    type Tag = tags::Set<tags::I32>;
    let v1 = [34, 1, 2];
    let v2 = [60, 1, 2, 0];

    let value = HashSet::from_iter([1]);
    assert_serde::<Tag, HashSet<i32>, _, _>(&value, v1, v2);

    let value = Value::I32Set(HashSet::from_iter([1]));
    assert_serde::<_, Value, _, _>(&value, v1, v2);

    let value = BTreeSet::from_iter([1]);
    assert_serde::<Tag, BTreeSet<i32>, _, _>(&value, v1, v2);
}

#[test]
fn test_u64_set() {
    type Tag = tags::Set<tags::U64>;
    let v1 = [35, 1, 2];
    let v2 = [61, 1, 2, 0];

    let value = HashSet::from_iter([2]);
    assert_serde::<Tag, HashSet<u64>, _, _>(&value, v1, v2);

    let value = Value::U64Set(HashSet::from_iter([2]));
    assert_serde::<_, Value, _, _>(&value, v1, v2);

    let value = BTreeSet::from_iter([2]);
    assert_serde::<Tag, BTreeSet<u64>, _, _>(&value, v1, v2);
}

#[test]
fn test_i64_set() {
    type Tag = tags::Set<tags::I64>;
    let v1 = [36, 1, 2];
    let v2 = [62, 1, 2, 0];

    let value = HashSet::from_iter([1]);
    assert_serde::<Tag, HashSet<i64>, _, _>(&value, v1, v2);

    let value = Value::I64Set(HashSet::from_iter([1]));
    assert_serde::<_, Value, _, _>(&value, v1, v2);

    let value = BTreeSet::from_iter([1]);
    assert_serde::<Tag, BTreeSet<i64>, _, _>(&value, v1, v2);
}

#[test]
fn test_string_set() {
    type Tag = tags::Set<tags::String>;
    let v1 = [37, 1, 2, b'3', b'4'];
    let v2 = [63, 1, 2, b'3', b'4', 0];

    let value = HashSet::from_iter(["34".to_owned()]);
    assert_serde::<Tag, HashSet<String>, _, _>(&value, v1, v2);

    let value = HashSet::from_iter(["34"]);
    assert_serialize::<Tag, HashSet<&str>, _>(&value, v2);

    let value = Value::StringSet(HashSet::from_iter(["34".to_owned()]));
    assert_serde::<_, Value, _, _>(&value, v1, v2);

    let value = BTreeSet::from_iter(["34".to_owned()]);
    assert_serde::<Tag, BTreeSet<String>, _, _>(&value, v1, v2);

    let value = BTreeSet::from_iter(["34"]);
    assert_serialize::<Tag, BTreeSet<&str>, _>(&value, v2);
}

#[test]
fn test_uuid_set() {
    type Tag = tags::Set<tags::Uuid>;
    let uuid = uuid!("81494c44-3bed-48e6-b078-1a93a1ae0e29");
    let v1 = [
        38, 1, 0x81, 0x49, 0x4c, 0x44, 0x3b, 0xed, 0x48, 0xe6, 0xb0, 0x78, 0x1a, 0x93, 0xa1, 0xae,
        0x0e, 0x29,
    ];
    let v2 = [
        64, 1, 0x81, 0x49, 0x4c, 0x44, 0x3b, 0xed, 0x48, 0xe6, 0xb0, 0x78, 0x1a, 0x93, 0xa1, 0xae,
        0x0e, 0x29, 0,
    ];

    let value = HashSet::from_iter([uuid]);
    assert_serde::<Tag, HashSet<Uuid>, _, _>(&value, v1, v2);

    let value = Value::UuidSet(HashSet::from_iter([uuid]));
    assert_serde::<_, Value, _, _>(&value, v1, v2);

    let value = BTreeSet::from_iter([uuid]);
    assert_serde::<Tag, BTreeSet<Uuid>, _, _>(&value, v1, v2);
}

#[test]
fn test_sender() {
    type Tag = tags::Sender<tags::Unit>;
    let uuid = uuid!("01234567-89ab-cdef-0246-8ace13579bdf");
    let serialized = [
        41, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x02, 0x46, 0x8a, 0xce, 0x13, 0x57,
        0x9b, 0xdf,
    ];

    let value = uuid;
    assert_serde::<Tag, Uuid, _, _>(&value, serialized, serialized);

    let value = Value::Sender(ChannelCookie(uuid));
    assert_serde::<_, Value, _, _>(&value, serialized, serialized);

    let value = ChannelCookie(uuid);
    assert_serde::<Tag, ChannelCookie, _, _>(&value, serialized, serialized);
}

#[test]
fn test_receiver() {
    type Tag = tags::Receiver<tags::Unit>;
    let uuid = uuid!("01234567-89ab-cdef-0246-8ace13579bdf");
    let serialized = [
        42, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x02, 0x46, 0x8a, 0xce, 0x13, 0x57,
        0x9b, 0xdf,
    ];

    let value = uuid;
    assert_serde::<Tag, Uuid, _, _>(&value, serialized, serialized);

    let value = Value::Receiver(ChannelCookie(uuid));
    assert_serde::<_, Value, _, _>(&value, serialized, serialized);

    let value = ChannelCookie(uuid);
    assert_serde::<Tag, ChannelCookie, _, _>(&value, serialized, serialized);
}

#[test]
fn test_result_ok() {
    type Tag = Result<tags::U8, tags::U8>;
    let serialized = [40, 0, 3, 1];

    let value = Ok(1);
    assert_serde::<Tag, Result<u8, u8>, _, _>(&value, serialized, serialized);
}

#[test]
fn test_result_err() {
    type Tag = Result<tags::U8, tags::U8>;
    let serialized = [40, 1, 3, 2];

    let value = Err(2);
    assert_serde::<Tag, Result<u8, u8>, _, _>(&value, serialized, serialized);
}

#[test]
fn test_serialize_too_deep() {
    let value = Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(
        Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(
            Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(0)))))))))),
        ))))))))))),
    )))))))))));

    assert_eq!(
        SerializedValue::serialize(value),
        Err(SerializeError::TooDeeplyNested)
    );
}

#[test]
fn test_deserialize_too_deep() {
    let serialized = SerializedValueSlice::new(&[
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 0,
    ]);

    assert_eq!(
        serialized.deserialize::<Value>(),
        Err(DeserializeError::TooDeeplyNested)
    );
}

#[test]
fn test_convert_too_deep() {
    let serialized = SerializedValueSlice::new(&[
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 0,
    ]);

    assert_eq!(
        serialized
            .convert(None, ProtocolVersion::V1_14)
            .unwrap_err(),
        ValueConversionError::Deserialize(DeserializeError::TooDeeplyNested),
    );
}

#[test]
fn test_struct() {
    type Tag = tags::Value;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct TestStruct {
        field1: u32,
        field2: Option<i32>,
    }

    impl Serialize<Tag> for TestStruct {
        fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
            serializer.serialize::<Tag>(&self)
        }
    }

    impl Serialize<Tag> for &TestStruct {
        fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
            let mut serializer = serializer.serialize_struct2()?;

            serializer.serialize::<tags::U32>(1u32, &self.field1)?;
            serializer.serialize_if_some::<tags::Option<tags::I32>>(2u32, &self.field2)?;

            serializer.finish()
        }
    }

    impl Deserialize<Tag> for TestStruct {
        fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
            let mut deserializer = deserializer.deserialize_struct()?;

            let mut field1 = None;
            let mut field2 = None;

            while let Some(deserializer) = deserializer.deserialize()? {
                match deserializer.id() {
                    1 => field1 = deserializer.deserialize::<tags::U32, _>().map(Some)?,
                    2 => field2 = deserializer.deserialize::<tags::Option<tags::I32>, _>()?,
                    _ => deserializer.skip()?,
                }
            }

            deserializer.finish(Self {
                field1: field1.ok_or(DeserializeError::InvalidSerialization)?,
                field2,
            })
        }
    }

    let v1 = [39, 2, 1, 7, 2, 2, 1, 8, 6];
    let v2 = [65, 1, 1, 7, 2, 1, 2, 1, 8, 6, 0];

    let value = TestStruct {
        field1: 2,
        field2: Some(3),
    };

    assert_serde::<Tag, TestStruct, _, _>(&value, v1, v2);
    assert_serialize::<Tag, &TestStruct, _>(&&value, v2);

    let v1 = [39, 1, 1, 7, 1];
    let v2 = [65, 1, 1, 7, 1, 0];

    let value = TestStruct {
        field1: 1,
        field2: None,
    };

    assert_serde::<Tag, TestStruct, _, _>(&value, v1, v2);
    assert_serialize::<Tag, &TestStruct, _>(&&value, v2);
}

#[test]
fn test_struct_fallback() {
    type Tag = tags::Value;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct TestStruct {
        field1: u32,
        fallback: UnknownFields,
    }

    impl Serialize<Tag> for TestStruct {
        fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
            serializer.serialize::<Tag>(&self)
        }
    }

    impl Serialize<Tag> for &TestStruct {
        fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
            let mut serializer =
                serializer.serialize_struct2_with_unknown_fields(&self.fallback)?;

            serializer.serialize::<tags::U32>(1u32, &self.field1)?;

            serializer.finish()
        }
    }

    impl Deserialize<Tag> for TestStruct {
        fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
            let mut deserializer = deserializer.deserialize_struct()?;

            let mut field1 = None;

            while let Some(deserializer) = deserializer.deserialize()? {
                match deserializer.id() {
                    1 => field1 = deserializer.deserialize::<tags::U32, _>().map(Some)?,
                    _ => deserializer.add_to_unknown_fields()?,
                }
            }

            deserializer.finish_with(|fallback| {
                Ok(Self {
                    field1: field1.ok_or(DeserializeError::InvalidSerialization)?,
                    fallback,
                })
            })
        }
    }

    let v1 = [39, 1, 1, 7, 1];
    let v2 = [65, 1, 1, 7, 1, 0];

    let value = TestStruct {
        field1: 1,
        fallback: UnknownFields::new(),
    };

    assert_serde::<Tag, TestStruct, _, _>(&value, v1, v2);
    assert_serialize::<Tag, &TestStruct, _>(&&value, v2);

    let v1 = [39, 2, 2, 1, 8, 4, 1, 7, 1];
    let v2 = [65, 1, 2, 1, 8, 4, 1, 1, 7, 1, 0];

    let mut fallback = UnknownFields::new();
    fallback.insert(2, SerializedValue::serialize(Some(2i32)).unwrap());

    let value = TestStruct {
        field1: 1,
        fallback,
    };

    assert_serde::<Tag, TestStruct, _, _>(&value, v1, v2);
    assert_serialize::<Tag, &TestStruct, _>(&&value, v2);
}

#[test]
fn test_enum() {
    type Tag = tags::Value;

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum TestEnum {
        Var1,
        Var2(u8),
    }

    impl Serialize<Tag> for TestEnum {
        fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
            serializer.serialize::<Tag>(&self)
        }
    }

    impl Serialize<Tag> for &TestEnum {
        fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
            match self {
                TestEnum::Var1 => serializer.serialize_unit_enum(1u32),
                TestEnum::Var2(value) => serializer.serialize_enum::<tags::U8>(2u32, value),
            }
        }
    }

    impl Deserialize<Tag> for TestEnum {
        fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
            let deserializer = deserializer.deserialize_enum()?;

            match deserializer.id() {
                1 => deserializer.deserialize_unit().map(|()| Self::Var1),
                2 => deserializer.deserialize::<tags::U8, _>().map(Self::Var2),
                _ => Err(DeserializeError::InvalidSerialization),
            }
        }
    }

    let serialized = [40, 1, 0];
    let value = TestEnum::Var1;
    assert_serde::<Tag, TestEnum, _, _>(&value, serialized, serialized);
    assert_serialize::<Tag, &TestEnum, _>(&&value, serialized);

    let serialized = [40, 2, 3, 4];
    let value = TestEnum::Var2(4);
    assert_serde::<Tag, TestEnum, _, _>(&value, serialized, serialized);
    assert_serialize::<Tag, &TestEnum, _>(&&value, serialized);
}

#[test]
fn test_enum_fallback() {
    type Tag = tags::Value;

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum TestEnum {
        Var1,
        Fallback(UnknownVariant),
    }

    impl Serialize<Tag> for TestEnum {
        fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
            serializer.serialize::<Tag>(&self)
        }
    }

    impl Serialize<Tag> for &TestEnum {
        fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
            match self {
                TestEnum::Var1 => serializer.serialize_unit_enum(1u32),
                TestEnum::Fallback(fallback) => serializer.serialize_unknown_variant(fallback),
            }
        }
    }

    impl Deserialize<Tag> for TestEnum {
        fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
            let deserializer = deserializer.deserialize_enum()?;

            match deserializer.id() {
                1 => deserializer.deserialize_unit().map(|()| Self::Var1),
                _ => deserializer.into_unknown_variant().map(Self::Fallback),
            }
        }
    }

    let serialized = [40, 1, 0];
    let value = TestEnum::Var1;
    assert_serde::<Tag, TestEnum, _, _>(&value, serialized, serialized);
    assert_serialize::<Tag, &TestEnum, _>(&&value, serialized);

    let serialized = [40, 2, 3, 4];

    let value = TestEnum::Fallback(UnknownVariant::new(
        2,
        SerializedValue::serialize(4u8).unwrap(),
    ));

    assert_serde::<Tag, TestEnum, _, _>(&value, serialized, serialized);
    assert_serialize::<Tag, &TestEnum, _>(&&value, serialized);
}
