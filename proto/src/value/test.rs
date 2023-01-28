use super::{ByteSlice, Bytes, Skip};
use crate::error::{DeserializeError, SerializeError};
use crate::generic_value::{Enum, Struct, Value};
use crate::ids::{
    ChannelCookie, ObjectCookie, ObjectId, ObjectUuid, ServiceCookie, ServiceId, ServiceUuid,
};
use crate::message::ChannelEnd;
use crate::serialized_value::{SerializedValue, SerializedValueSlice};
use crate::value_deserializer::{Deserialize, Deserializer};
use crate::value_serializer::{Serialize, Serializer};
use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, LinkedList, VecDeque};
use std::fmt::Debug;
use std::{f32, f64};
use uuid::uuid;

fn assert_serialize_eq<T: Serialize + ?Sized, B: AsRef<[u8]>>(value: &T, expected: B) {
    let serialized_value = SerializedValue::serialize(value).unwrap();
    assert_eq!(serialized_value, *expected.as_ref());
}

fn assert_deserialize_eq<T: Deserialize + PartialEq + Debug, B: AsRef<[u8]>>(
    expected: &T,
    serialized: B,
) {
    let serialized_value = SerializedValueSlice::new(&serialized);

    // Actual deserialization
    assert_eq!(*expected, serialized_value.deserialize().unwrap());

    // skip
    let mut buf = serialized.as_ref();
    Deserializer::new(&mut buf).skip().unwrap();
    assert_eq!(*buf, []);
    assert_eq!(serialized_value.deserialize(), Ok(Skip));

    // len
    let mut buf = serialized.as_ref();
    let len = Deserializer::new(&mut buf).len().unwrap();
    assert_eq!(len, buf.len());
}

#[test]
fn test_none() {
    let serialized = [0];
    #[allow(clippy::let_unit_value)]
    let value = ();
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    let value = Value::None;
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
}

#[test]
fn test_some() {
    let serialized = [1, 0];
    let value = Some(());
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    let value = Value::Some(Box::new(Value::None));
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    let value = Some(Option::<()>::None);
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
}

#[test]
fn test_bool() {
    let serialized = [2, 0];
    let value1 = false;
    assert_serialize_eq(&value1, serialized);
    assert_deserialize_eq(&value1, serialized);
    let value2 = Value::Bool(value1);
    assert_serialize_eq(&value2, serialized);
    assert_deserialize_eq(&value2, serialized);

    let serialized = [2, 1];
    let value1 = true;
    assert_serialize_eq(&value1, serialized);
    assert_deserialize_eq(&value1, serialized);
    let value2 = Value::Bool(value1);
    assert_serialize_eq(&value2, serialized);
    assert_deserialize_eq(&value2, serialized);

    // Any value != 0 deserializes to true.
    let value = true;
    let serialized = [2, 2];
    assert_deserialize_eq(&value, serialized);
}

#[test]
fn test_u8() {
    let serialized = [3, 0];
    let value1 = 0u8;
    assert_serialize_eq(&value1, serialized);
    assert_deserialize_eq(&value1, serialized);
    let value2 = Value::U8(value1);
    assert_serialize_eq(&value2, serialized);
    assert_deserialize_eq(&value2, serialized);

    let serialized = [3, 255];
    let value1 = 255u8;
    assert_serialize_eq(&value1, serialized);
    assert_deserialize_eq(&value1, serialized);
    let value2 = Value::U8(value1);
    assert_serialize_eq(&value2, serialized);
    assert_deserialize_eq(&value2, serialized);
}

#[test]
fn test_i8() {
    let serialized = [4, 0];
    let value1 = 0i8;
    assert_serialize_eq(&value1, serialized);
    assert_deserialize_eq(&value1, serialized);
    let value2 = Value::I8(value1);
    assert_serialize_eq(&value2, serialized);
    assert_deserialize_eq(&value2, serialized);

    let serialized = [4, 1];
    let value1 = 1i8;
    assert_serialize_eq(&value1, serialized);
    assert_deserialize_eq(&value1, serialized);
    let value2 = Value::I8(value1);
    assert_serialize_eq(&value2, serialized);
    assert_deserialize_eq(&value2, serialized);

    let serialized = [4, 255];
    let value1 = -1i8;
    assert_serialize_eq(&value1, serialized);
    assert_deserialize_eq(&value1, serialized);
    let value2 = Value::I8(value1);
    assert_serialize_eq(&value2, serialized);
    assert_deserialize_eq(&value2, serialized);

    let serialized = [4, 127];
    let value1 = 127i8;
    assert_serialize_eq(&value1, serialized);
    assert_deserialize_eq(&value1, serialized);
    let value2 = Value::I8(value1);
    assert_serialize_eq(&value2, serialized);
    assert_deserialize_eq(&value2, serialized);

    let serialized = [4, 128];
    let value1 = -128i8;
    assert_serialize_eq(&value1, serialized);
    assert_deserialize_eq(&value1, serialized);
    let value2 = Value::I8(value1);
    assert_serialize_eq(&value2, serialized);
    assert_deserialize_eq(&value2, serialized);
}

#[test]
fn test_u16() {
    let serialized = [5, 0];
    let value1 = 0u16;
    assert_serialize_eq(&value1, serialized);
    assert_deserialize_eq(&value1, serialized);
    let value2 = Value::U16(value1);
    assert_serialize_eq(&value2, serialized);
    assert_deserialize_eq(&value2, serialized);

    let serialized = [5, 255, 255, 255];
    let value1 = u16::MAX;
    assert_serialize_eq(&value1, serialized);
    assert_deserialize_eq(&value1, serialized);
    let value2 = Value::U16(value1);
    assert_serialize_eq(&value2, serialized);
    assert_deserialize_eq(&value2, serialized);
}

#[test]
fn test_i16() {
    let serialized = [6, 0];
    let value1 = 0i16;
    assert_serialize_eq(&value1, serialized);
    assert_deserialize_eq(&value1, serialized);
    let value2 = Value::I16(value1);
    assert_serialize_eq(&value2, serialized);
    assert_deserialize_eq(&value2, serialized);

    let serialized = [6, 2];
    let value1 = 1i16;
    assert_serialize_eq(&value1, serialized);
    assert_deserialize_eq(&value1, serialized);
    let value2 = Value::I16(value1);
    assert_serialize_eq(&value2, serialized);
    assert_deserialize_eq(&value2, serialized);

    let serialized = [6, 1];
    let value1 = -1i16;
    assert_serialize_eq(&value1, serialized);
    assert_deserialize_eq(&value1, serialized);
    let value2 = Value::I16(value1);
    assert_serialize_eq(&value2, serialized);
    assert_deserialize_eq(&value2, serialized);

    let serialized = [6, 255, 254, 255];
    let value1 = i16::MAX;
    assert_serialize_eq(&value1, serialized);
    assert_deserialize_eq(&value1, serialized);
    let value2 = Value::I16(value1);
    assert_serialize_eq(&value2, serialized);
    assert_deserialize_eq(&value2, serialized);

    let serialized = [6, 255, 255, 255];
    let value1 = i16::MIN;
    assert_serialize_eq(&value1, serialized);
    assert_deserialize_eq(&value1, serialized);
    let value2 = Value::I16(value1);
    assert_serialize_eq(&value2, serialized);
    assert_deserialize_eq(&value2, serialized);
}

#[test]
fn test_u32() {
    let serialized = [7, 0];
    let value1 = 0u32;
    assert_serialize_eq(&value1, serialized);
    assert_deserialize_eq(&value1, serialized);
    let value2 = Value::U32(value1);
    assert_serialize_eq(&value2, serialized);
    assert_deserialize_eq(&value2, serialized);

    let serialized = [7, 255, 255, 255, 255, 255];
    let value1 = u32::MAX;
    assert_serialize_eq(&value1, serialized);
    assert_deserialize_eq(&value1, serialized);
    let value2 = Value::U32(value1);
    assert_serialize_eq(&value2, serialized);
    assert_deserialize_eq(&value2, serialized);
}

#[test]
fn test_i32() {
    let serialized = [8, 0];
    let value1 = 0i32;
    assert_serialize_eq(&value1, serialized);
    assert_deserialize_eq(&value1, serialized);
    let value2 = Value::I32(value1);
    assert_serialize_eq(&value2, serialized);
    assert_deserialize_eq(&value2, serialized);

    let serialized = [8, 2];
    let value1 = 1i32;
    assert_serialize_eq(&value1, serialized);
    assert_deserialize_eq(&value1, serialized);
    let value2 = Value::I32(value1);
    assert_serialize_eq(&value2, serialized);
    assert_deserialize_eq(&value2, serialized);

    let serialized = [8, 1];
    let value1 = -1i32;
    assert_serialize_eq(&value1, serialized);
    assert_deserialize_eq(&value1, serialized);
    let value2 = Value::I32(value1);
    assert_serialize_eq(&value2, serialized);
    assert_deserialize_eq(&value2, serialized);

    let serialized = [8, 255, 254, 255, 255, 255];
    let value1 = i32::MAX;
    assert_serialize_eq(&value1, serialized);
    assert_deserialize_eq(&value1, serialized);
    let value2 = Value::I32(value1);
    assert_serialize_eq(&value2, serialized);
    assert_deserialize_eq(&value2, serialized);

    let serialized = [8, 255, 255, 255, 255, 255];
    let value1 = i32::MIN;
    assert_serialize_eq(&value1, serialized);
    assert_deserialize_eq(&value1, serialized);
    let value2 = Value::I32(value1);
    assert_serialize_eq(&value2, serialized);
    assert_deserialize_eq(&value2, serialized);
}

#[test]
fn test_u64() {
    let serialized = [9, 0];
    let value1 = 0u64;
    assert_serialize_eq(&value1, serialized);
    assert_deserialize_eq(&value1, serialized);
    let value2 = Value::U64(value1);
    assert_serialize_eq(&value2, serialized);
    assert_deserialize_eq(&value2, serialized);

    let serialized = [9, 255, 255, 255, 255, 255, 255, 255, 255, 255];
    let value1 = u64::MAX;
    assert_serialize_eq(&value1, serialized);
    assert_deserialize_eq(&value1, serialized);
    let value2 = Value::U64(value1);
    assert_serialize_eq(&value2, serialized);
    assert_deserialize_eq(&value2, serialized);
}

#[test]
fn test_i64() {
    let serialized = [10, 0];
    let value1 = 0i64;
    assert_serialize_eq(&value1, serialized);
    assert_deserialize_eq(&value1, serialized);
    let value2 = Value::I64(value1);
    assert_serialize_eq(&value2, serialized);
    assert_deserialize_eq(&value2, serialized);

    let serialized = [10, 2];
    let value1 = 1i64;
    assert_serialize_eq(&value1, serialized);
    assert_deserialize_eq(&value1, serialized);
    let value2 = Value::I64(value1);
    assert_serialize_eq(&value2, serialized);
    assert_deserialize_eq(&value2, serialized);

    let serialized = [10, 1];
    let value1 = -1i64;
    assert_serialize_eq(&value1, serialized);
    assert_deserialize_eq(&value1, serialized);
    let value2 = Value::I64(value1);
    assert_serialize_eq(&value2, serialized);
    assert_deserialize_eq(&value2, serialized);

    let serialized = [10, 255, 254, 255, 255, 255, 255, 255, 255, 255];
    let value1 = i64::MAX;
    assert_serialize_eq(&value1, serialized);
    assert_deserialize_eq(&value1, serialized);
    let value2 = Value::I64(value1);
    assert_serialize_eq(&value2, serialized);
    assert_deserialize_eq(&value2, serialized);

    let serialized = [10, 255, 255, 255, 255, 255, 255, 255, 255, 255];
    let value1 = i64::MIN;
    assert_serialize_eq(&value1, serialized);
    assert_deserialize_eq(&value1, serialized);
    let value2 = Value::I64(value1);
    assert_serialize_eq(&value2, serialized);
    assert_deserialize_eq(&value2, serialized);
}

#[test]
fn test_f32() {
    let serialized = [11, 0, 0, 0, 0];
    let value1 = 0f32;
    assert_serialize_eq(&value1, serialized);
    assert_deserialize_eq(&value1, serialized);
    let value2 = Value::F32(value1);
    assert_serialize_eq(&value2, serialized);
    assert_deserialize_eq(&value2, serialized);

    let serialized = [11, 219, 15, 73, 64];
    let value1 = f32::consts::PI;
    assert_serialize_eq(&value1, serialized);
    assert_deserialize_eq(&value1, serialized);
    let value2 = Value::F32(value1);
    assert_serialize_eq(&value2, serialized);
    assert_deserialize_eq(&value2, serialized);
}

#[test]
fn test_f64() {
    let serialized = [12, 0, 0, 0, 0, 0, 0, 0, 0];
    let value1 = 0f64;
    assert_serialize_eq(&value1, serialized);
    assert_deserialize_eq(&value1, serialized);
    let value2 = Value::F64(value1);
    assert_serialize_eq(&value2, serialized);
    assert_deserialize_eq(&value2, serialized);

    let serialized = [12, 24, 45, 68, 84, 251, 33, 9, 64];
    let value1 = f64::consts::PI;
    assert_serialize_eq(&value1, serialized);
    assert_deserialize_eq(&value1, serialized);
    let value2 = Value::F64(value1);
    assert_serialize_eq(&value2, serialized);
    assert_deserialize_eq(&value2, serialized);
}

#[test]
fn test_string() {
    let serialized = [13, 4, b'a', b'b', b'c', b'd'];
    let value1 = "abcd".to_owned();
    assert_serialize_eq(&value1, serialized);
    assert_deserialize_eq(&value1, serialized);
    let value2 = Value::String(value1);
    assert_serialize_eq(&value2, serialized);
    assert_deserialize_eq(&value2, serialized);

    let serialized = [13, 0];
    let value1 = String::new();
    assert_serialize_eq(&value1, serialized);
    assert_deserialize_eq(&value1, serialized);
    let value2 = Value::String(value1);
    assert_serialize_eq(&value2, serialized);
    assert_deserialize_eq(&value2, serialized);

    let serialized = [13, 6, 195, 164, 195, 182, 195, 188];
    let value1 = "äöü".to_owned();
    assert_serialize_eq(&value1, serialized);
    assert_deserialize_eq(&value1, serialized);
    let value2 = Value::String(value1);
    assert_serialize_eq(&value2, serialized);
    assert_deserialize_eq(&value2, serialized);

    let value = "abcd";
    let serialized = [13, 4, b'a', b'b', b'c', b'd'];
    assert_serialize_eq(&value, serialized);
}

#[test]
fn test_uuid() {
    let serialized = [
        14, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x02, 0x46, 0x8a, 0xce, 0x13, 0x57,
        0x9b, 0xdf,
    ];
    let value1 = uuid!("01234567-89ab-cdef-0246-8ace13579bdf");
    assert_serialize_eq(&value1, serialized);
    assert_deserialize_eq(&value1, serialized);
    let value2 = Value::Uuid(value1);
    assert_serialize_eq(&value2, serialized);
    assert_deserialize_eq(&value2, serialized);
}

#[test]
fn test_vec() {
    let serialized = [17, 2, 3, 7, 3, 8];
    let value1 = vec![7u8, 8];
    assert_serialize_eq(&value1, serialized);
    assert_deserialize_eq(&value1, serialized);
    let value2 = Value::Vec(vec![Value::U8(7), Value::U8(8)]);
    assert_serialize_eq(&value2, serialized);
    assert_deserialize_eq(&value2, serialized);
    let value3 = VecDeque::from(value1.clone());
    assert_serialize_eq(&value3, serialized);
    assert_deserialize_eq(&value3, serialized);
    let value4 = LinkedList::from_iter(value1.iter().copied());
    assert_serialize_eq(&value4, serialized);
    assert_deserialize_eq(&value4, serialized);
    let value5 = [7u8, 8];
    assert_serialize_eq(&value5, serialized);
    assert_serialize_eq(&value5[..], serialized);
    assert_deserialize_eq(&value5, serialized);

    let value = Value::Vec(vec![Value::None, Value::U8(4)]);
    let serialized = [17, 2, 0, 3, 4];
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
}

#[test]
fn test_bytes() {
    let serialized = [18, 3, 1, 2, 3];
    let value1 = Bytes::new([1, 2, 3]);
    assert_serialize_eq(&value1, serialized);
    assert_deserialize_eq(&value1, serialized);
    let value2 = Value::Bytes(value1.0.clone());
    assert_serialize_eq(&value2, serialized);
    assert_deserialize_eq(&value2, serialized);
    let value3 = bytes::Bytes::from(value1.0.clone());
    assert_serialize_eq(&value3, serialized);
    assert_deserialize_eq(&value3, serialized);
    let value4 = bytes::BytesMut::from_iter(value1.0.iter());
    assert_serialize_eq(&value4, serialized);
    assert_deserialize_eq(&value4, serialized);
    let value5 = ByteSlice::new(&[1, 2, 3]);
    assert_serialize_eq(&value5, serialized);
}

#[test]
fn test_bytes_partial_deserialize() {
    #[derive(Debug, PartialEq)]
    struct Parts([u8; 3], [u8; 2]);

    impl Serialize for Parts {
        fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
            let mut serializer = serializer.serialize_bytes(6)?;
            serializer.serialize(&self.0)?;
            serializer.serialize(&[3])?;
            serializer.serialize(&self.1)?;
            serializer.finish()
        }
    }

    impl Deserialize for Parts {
        fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
            let mut deserializer = deserializer.deserialize_bytes()?;

            let mut a = <[u8; 3]>::default();
            let mut b = <[u8; 2]>::default();

            deserializer.deserialize(&mut a)?;
            deserializer.skip(1)?;
            deserializer.deserialize(&mut b)?;

            Ok(Self(a, b))
        }
    }

    let serialized = [18, 6, 0, 1, 2, 3, 4, 5];
    let value = Parts([0, 1, 2], [4, 5]);
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
}

#[test]
fn test_u8_map() {
    let serialized1 = [19, 2, 0, 3, 1, 2, 3, 3];
    let serialized2 = [19, 2, 2, 3, 3, 0, 3, 1];

    let value = HashMap::<u8, u8>::from_iter([(0, 1), (2, 3)]);
    let mut buf = bytes::BytesMut::new();
    value.serialize(Serializer::new(&mut buf)).unwrap();
    assert!((buf[..] == serialized1) || (buf[..] == serialized2));
    assert_deserialize_eq(&value, serialized1);
    assert_deserialize_eq(&value, serialized2);

    let value = Value::U8Map(HashMap::from_iter([(0, Value::U8(1)), (2, Value::U8(3))]));
    let mut buf = bytes::BytesMut::new();
    value.serialize(Serializer::new(&mut buf)).unwrap();
    assert!((buf[..] == serialized1) || (buf[..] == serialized2));
    assert_deserialize_eq(&value, serialized1);
    assert_deserialize_eq(&value, serialized2);

    let serialized = [19, 2, 0, 3, 1, 2, 3, 3];
    let value = BTreeMap::<u8, u8>::from_iter([(0, 1), (2, 3)]);
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
}

#[test]
fn test_i8_map() {
    let serialized = [20, 1, 2, 3, 4];
    let value1 = HashMap::<i8, u8>::from_iter([(2, 4)]);
    assert_serialize_eq(&value1, serialized);
    assert_deserialize_eq(&value1, serialized);
    let value2 = BTreeMap::from_iter([(2i8, 4u8)]);
    assert_serialize_eq(&value2, serialized);
    assert_deserialize_eq(&value2, serialized);
    let value3 = Value::I8Map(HashMap::from_iter([(2, Value::U8(4))]));
    assert_serialize_eq(&value3, serialized);
    assert_deserialize_eq(&value3, serialized);
}

#[test]
fn test_u16_map() {
    let serialized = [21, 1, 2, 3, 4];
    let value1 = HashMap::<u16, u8>::from_iter([(2, 4)]);
    assert_serialize_eq(&value1, serialized);
    assert_deserialize_eq(&value1, serialized);
    let value2 = BTreeMap::from_iter([(2u16, 4u8)]);
    assert_serialize_eq(&value2, serialized);
    assert_deserialize_eq(&value2, serialized);
    let value3 = Value::U16Map(HashMap::from_iter([(2, Value::U8(4))]));
    assert_serialize_eq(&value3, serialized);
    assert_deserialize_eq(&value3, serialized);
}

#[test]
fn test_i16_map() {
    let serialized = [22, 1, 4, 3, 4];
    let value1 = HashMap::<i16, u8>::from_iter([(2, 4)]);
    assert_serialize_eq(&value1, serialized);
    assert_deserialize_eq(&value1, serialized);
    let value2 = BTreeMap::from_iter([(2i16, 4u8)]);
    assert_serialize_eq(&value2, serialized);
    assert_deserialize_eq(&value2, serialized);
    let value3 = Value::I16Map(HashMap::from_iter([(2, Value::U8(4))]));
    assert_serialize_eq(&value3, serialized);
    assert_deserialize_eq(&value3, serialized);
}

#[test]
fn test_u32_map() {
    let serialized = [23, 1, 2, 3, 4];
    let value1 = HashMap::<u32, u8>::from_iter([(2, 4)]);
    assert_serialize_eq(&value1, serialized);
    assert_deserialize_eq(&value1, serialized);
    let value2 = BTreeMap::from_iter([(2u32, 4u8)]);
    assert_serialize_eq(&value2, serialized);
    assert_deserialize_eq(&value2, serialized);
    let value3 = Value::U32Map(HashMap::from_iter([(2, Value::U8(4))]));
    assert_serialize_eq(&value3, serialized);
    assert_deserialize_eq(&value3, serialized);
}

#[test]
fn test_i32_map() {
    let serialized = [24, 1, 4, 3, 4];
    let value1 = HashMap::<i32, u8>::from_iter([(2, 4)]);
    assert_serialize_eq(&value1, serialized);
    assert_deserialize_eq(&value1, serialized);
    let value2 = BTreeMap::from_iter([(2i32, 4u8)]);
    assert_serialize_eq(&value2, serialized);
    assert_deserialize_eq(&value2, serialized);
    let value3 = Value::I32Map(HashMap::from_iter([(2, Value::U8(4))]));
    assert_serialize_eq(&value3, serialized);
    assert_deserialize_eq(&value3, serialized);
}

#[test]
fn test_u64_map() {
    let serialized = [25, 1, 2, 3, 4];
    let value1 = HashMap::<u64, u8>::from_iter([(2, 4)]);
    assert_serialize_eq(&value1, serialized);
    assert_deserialize_eq(&value1, serialized);
    let value2 = BTreeMap::from_iter([(2u64, 4u8)]);
    assert_serialize_eq(&value2, serialized);
    assert_deserialize_eq(&value2, serialized);
    let value3 = Value::U64Map(HashMap::from_iter([(2, Value::U8(4))]));
    assert_serialize_eq(&value3, serialized);
    assert_deserialize_eq(&value3, serialized);
}

#[test]
fn test_i64_map() {
    let serialized = [26, 1, 4, 3, 4];
    let value1 = HashMap::<i64, u8>::from_iter([(2, 4)]);
    assert_serialize_eq(&value1, serialized);
    assert_deserialize_eq(&value1, serialized);
    let value2 = BTreeMap::from_iter([(2i64, 4u8)]);
    assert_serialize_eq(&value2, serialized);
    assert_deserialize_eq(&value2, serialized);
    let value3 = Value::I64Map(HashMap::from_iter([(2, Value::U8(4))]));
    assert_serialize_eq(&value3, serialized);
    assert_deserialize_eq(&value3, serialized);
}

#[test]
fn test_string_map() {
    let serialized = [27, 1, 2, b'3', b'4', 5, 6];
    let value1 = HashMap::<String, u16>::from_iter([("34".to_owned(), 6)]);
    assert_serialize_eq(&value1, serialized);
    assert_deserialize_eq(&value1, serialized);
    let value2 = HashMap::<&str, u16>::from_iter([("34", 6)]);
    assert_serialize_eq(&value2, serialized);
    let value3 = BTreeMap::from_iter([("34".to_owned(), 6u16)]);
    assert_serialize_eq(&value3, serialized);
    assert_deserialize_eq(&value3, serialized);
    let value4 = BTreeMap::from_iter([("34", 6u16)]);
    assert_serialize_eq(&value4, serialized);
    let value5 = Value::StringMap(HashMap::from_iter([("34".to_owned(), Value::U16(6))]));
    assert_serialize_eq(&value5, serialized);
    assert_deserialize_eq(&value5, serialized);
}

#[test]
fn test_uuid_map() {
    let uuid = uuid!("81494c44-3bed-48e6-b078-1a93a1ae0e29");
    let serialized = [
        28, 1, 0x81, 0x49, 0x4c, 0x44, 0x3b, 0xed, 0x48, 0xe6, 0xb0, 0x78, 0x1a, 0x93, 0xa1, 0xae,
        0x0e, 0x29, 3, 0,
    ];
    let value1 = HashMap::<_, _>::from_iter([(uuid, 0u8)]);
    assert_serialize_eq(&value1, serialized);
    assert_deserialize_eq(&value1, serialized);
    let value2 = BTreeMap::from_iter([(uuid, 0u8)]);
    assert_serialize_eq(&value2, serialized);
    assert_deserialize_eq(&value2, serialized);
    let value3 = Value::UuidMap(HashMap::<_, _>::from_iter([(uuid, Value::U8(0))]));
    assert_serialize_eq(&value3, serialized);
    assert_deserialize_eq(&value3, serialized);
}

#[test]
fn test_u8_set() {
    let serialized1 = [29, 2, 3, 4];
    let serialized2 = [29, 2, 4, 3];

    let value1 = HashSet::<_>::from_iter([3u8, 4]);
    let mut buf = bytes::BytesMut::new();
    value1.serialize(Serializer::new(&mut buf)).unwrap();
    assert!((buf[..] == serialized1) || (buf[..] == serialized2));
    assert_deserialize_eq(&value1, serialized1);
    assert_deserialize_eq(&value1, serialized2);

    let value2 = Value::U8Set(value1);
    let mut buf = bytes::BytesMut::new();
    value2.serialize(Serializer::new(&mut buf)).unwrap();
    assert!((buf[..] == serialized1) || (buf[..] == serialized2));
    assert_deserialize_eq(&value2, serialized1);
    assert_deserialize_eq(&value2, serialized2);

    let serialized = [29, 2, 3, 4];
    let value3 = BTreeSet::from_iter([4u8, 3]);
    assert_serialize_eq(&value3, serialized);
    assert_deserialize_eq(&value3, serialized);
}

#[test]
fn test_i8_set() {
    let serialized = [30, 1, 2];
    let value1 = HashSet::<_>::from_iter([2i8]);
    assert_serialize_eq(&value1, serialized);
    assert_deserialize_eq(&value1, serialized);
    let value2 = BTreeSet::from_iter([2i8]);
    assert_serialize_eq(&value2, serialized);
    assert_deserialize_eq(&value2, serialized);
    let value3 = Value::I8Set(HashSet::from_iter([2]));
    assert_serialize_eq(&value3, serialized);
    assert_deserialize_eq(&value3, serialized);
}

#[test]
fn test_u16_set() {
    let serialized = [31, 1, 2];
    let value1 = HashSet::<_>::from_iter([2u16]);
    assert_serialize_eq(&value1, serialized);
    assert_deserialize_eq(&value1, serialized);
    let value2 = BTreeSet::from_iter([2u16]);
    assert_serialize_eq(&value2, serialized);
    assert_deserialize_eq(&value2, serialized);
    let value3 = Value::U16Set(HashSet::from_iter([2]));
    assert_serialize_eq(&value3, serialized);
    assert_deserialize_eq(&value3, serialized);
}

#[test]
fn test_i16_set() {
    let serialized = [32, 1, 2];
    let value1 = HashSet::<_>::from_iter([1i16]);
    assert_serialize_eq(&value1, serialized);
    assert_deserialize_eq(&value1, serialized);
    let value2 = BTreeSet::from_iter([1i16]);
    assert_serialize_eq(&value2, serialized);
    assert_deserialize_eq(&value2, serialized);
    let value3 = Value::I16Set(HashSet::from_iter([1]));
    assert_serialize_eq(&value3, serialized);
    assert_deserialize_eq(&value3, serialized);
}

#[test]
fn test_u32_set() {
    let serialized = [33, 1, 2];
    let value1 = HashSet::<_>::from_iter([2u32]);
    assert_serialize_eq(&value1, serialized);
    assert_deserialize_eq(&value1, serialized);
    let value2 = BTreeSet::from_iter([2u32]);
    assert_serialize_eq(&value2, serialized);
    assert_deserialize_eq(&value2, serialized);
    let value3 = Value::U32Set(HashSet::from_iter([2]));
    assert_serialize_eq(&value3, serialized);
    assert_deserialize_eq(&value3, serialized);
}

#[test]
fn test_i32_set() {
    let serialized = [34, 1, 2];
    let value1 = HashSet::<_>::from_iter([1i32]);
    assert_serialize_eq(&value1, serialized);
    assert_deserialize_eq(&value1, serialized);
    let value2 = BTreeSet::from_iter([1i32]);
    assert_serialize_eq(&value2, serialized);
    assert_deserialize_eq(&value2, serialized);
    let value3 = Value::I32Set(HashSet::from_iter([1]));
    assert_serialize_eq(&value3, serialized);
    assert_deserialize_eq(&value3, serialized);
}

#[test]
fn test_u64_set() {
    let serialized = [35, 1, 2];
    let value1 = HashSet::<_>::from_iter([2u64]);
    assert_serialize_eq(&value1, serialized);
    assert_deserialize_eq(&value1, serialized);
    let value2 = BTreeSet::from_iter([2u64]);
    assert_serialize_eq(&value2, serialized);
    assert_deserialize_eq(&value2, serialized);
    let value3 = Value::U64Set(HashSet::from_iter([2]));
    assert_serialize_eq(&value3, serialized);
    assert_deserialize_eq(&value3, serialized);
}

#[test]
fn test_i64_set() {
    let serialized = [36, 1, 2];
    let value1 = HashSet::<_>::from_iter([1i64]);
    assert_serialize_eq(&value1, serialized);
    assert_deserialize_eq(&value1, serialized);
    let value2 = BTreeSet::from_iter([1i64]);
    assert_serialize_eq(&value2, serialized);
    assert_deserialize_eq(&value2, serialized);
    let value3 = Value::I64Set(HashSet::from_iter([1]));
    assert_serialize_eq(&value3, serialized);
    assert_deserialize_eq(&value3, serialized);
}

#[test]
fn test_string_set() {
    let serialized = [37, 1, 2, b'3', b'4'];
    let value1 = HashSet::<_>::from_iter(["34".to_owned()]);
    assert_serialize_eq(&value1, serialized);
    assert_deserialize_eq(&value1, serialized);
    let value2 = HashSet::<_>::from_iter(["34"]);
    assert_serialize_eq(&value2, serialized);
    let value3 = BTreeSet::from_iter(["34".to_owned()]);
    assert_serialize_eq(&value3, serialized);
    assert_deserialize_eq(&value3, serialized);
    let value4 = BTreeSet::from_iter(["34"]);
    assert_serialize_eq(&value4, serialized);
    let value5 = Value::StringSet(HashSet::from_iter(["34".to_owned()]));
    assert_serialize_eq(&value5, serialized);
    assert_deserialize_eq(&value5, serialized);
}

#[test]
fn test_uuid_set() {
    let uuid = uuid!("81494c44-3bed-48e6-b078-1a93a1ae0e29");
    let serialized = [
        38, 1, 0x81, 0x49, 0x4c, 0x44, 0x3b, 0xed, 0x48, 0xe6, 0xb0, 0x78, 0x1a, 0x93, 0xa1, 0xae,
        0x0e, 0x29,
    ];
    let value1 = HashSet::<_>::from_iter([uuid]);
    assert_serialize_eq(&value1, serialized);
    assert_deserialize_eq(&value1, serialized);
    let value2 = BTreeSet::from_iter([uuid]);
    assert_serialize_eq(&value2, serialized);
    assert_deserialize_eq(&value2, serialized);
    let value3 = Value::UuidSet(HashSet::<_>::from_iter([uuid]));
    assert_serialize_eq(&value3, serialized);
    assert_deserialize_eq(&value3, serialized);
}

#[derive(Debug, PartialEq, Eq)]
struct TestStruct {
    a: u8,
    b: Option<String>,
}

impl Serialize for TestStruct {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct(2)?;
        serializer
            .serialize_field(0, &self.a)?
            .serialize_field(1, &self.b)?;
        serializer.finish()
    }
}

impl Deserialize for TestStruct {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let mut deserializer = deserializer.deserialize_struct()?;

        let mut a = None;
        let mut b = None;

        while deserializer.has_more_fields() {
            let deserializer = deserializer.deserialize_field()?;
            match deserializer.id() {
                0 => a = Some(deserializer.deserialize()?),
                1 => b = deserializer.deserialize()?,
                _ => return Err(DeserializeError::InvalidSerialization),
            }
        }

        Ok(Self {
            a: a.ok_or(DeserializeError::InvalidSerialization)?,
            b,
        })
    }
}

#[test]
fn test_struct() {
    let serialized = [39, 2, 0, 3, 4, 1, 0];
    let value = TestStruct { a: 4, b: None };
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);

    let serialized = [39, 2, 0, 3, 4, 1, 1, 13, 1, b'a'];
    let value = TestStruct {
        a: 4,
        b: Some("a".to_owned()),
    };
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);

    let serialized = [39, 1, 0, 3, 4];
    let value1 = Value::Struct(Struct(HashMap::from_iter([(0, Value::U8(4))])));
    assert_serialize_eq(&value1, serialized);
    assert_deserialize_eq(&value1, serialized);
    let value2 = Value::Struct(Struct(HashMap::from_iter([
        (0, Value::U8(4)),
        (1, Value::None),
    ])));
    assert_serialize_eq(&value2, serialized);
}

#[derive(Debug, PartialEq, Eq)]
enum TestEnum {
    A(u8),
    B(Option<String>),
}

impl Serialize for TestEnum {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        match self {
            Self::A(value) => serializer.serialize_enum(0, value),
            Self::B(value) => serializer.serialize_enum(1, value),
        }
    }
}

impl Deserialize for TestEnum {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let deserializer = deserializer.deserialize_enum()?;
        match deserializer.variant() {
            0 => Ok(Self::A(deserializer.deserialize()?)),
            1 => Ok(Self::B(deserializer.deserialize()?)),
            _ => Err(DeserializeError::InvalidSerialization),
        }
    }
}

#[test]
fn test_enum() {
    let serialized = [40, 0, 3, 4];
    let value1 = TestEnum::A(4);
    assert_serialize_eq(&value1, serialized);
    assert_deserialize_eq(&value1, serialized);
    let value2 = Value::Enum(Box::new(Enum::new(0, Value::U8(4))));
    assert_serialize_eq(&value2, serialized);
    assert_deserialize_eq(&value2, serialized);

    let serialized = [40, 1, 0];
    let value = TestEnum::B(None);
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    let value = Value::Enum(Box::new(Enum::new(1, Value::None)));
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);

    let serialized = [40, 1, 1, 13, 1, b'a'];
    let value = TestEnum::B(Some("a".to_owned()));
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    let value = Value::Enum(Box::new(Enum::new(
        1,
        Value::Some(Box::new(Value::String("a".to_owned()))),
    )));
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
}

#[test]
fn test_sender() {
    let channel_cookie = ChannelCookie(uuid!("81494c44-3bed-48e6-b078-1a93a1ae0e29"));
    let serialized = [
        41, 0x81, 0x49, 0x4c, 0x44, 0x3b, 0xed, 0x48, 0xe6, 0xb0, 0x78, 0x1a, 0x93, 0xa1, 0xae,
        0x0e, 0x29,
    ];
    let value = Value::Sender(channel_cookie);
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
}

#[test]
fn test_receiver() {
    let channel_cookie = ChannelCookie(uuid!("81494c44-3bed-48e6-b078-1a93a1ae0e29"));
    let serialized = [
        42, 0x81, 0x49, 0x4c, 0x44, 0x3b, 0xed, 0x48, 0xe6, 0xb0, 0x78, 0x1a, 0x93, 0xa1, 0xae,
        0x0e, 0x29,
    ];
    let value = Value::Receiver(channel_cookie);
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
}

#[test]
fn test_object_id() {
    let serialized = [
        15, 0x81, 0x49, 0x4c, 0x44, 0x3b, 0xed, 0x48, 0xe6, 0xb0, 0x78, 0x1a, 0x93, 0xa1, 0xae,
        0x0e, 0x29, 0xa2, 0x98, 0x85, 0xa9, 0x02, 0x12, 0x49, 0x40, 0x96, 0x4f, 0xe7, 0x30, 0x21,
        0x31, 0x71, 0x4b,
    ];

    let value1 = ObjectId::new(
        ObjectUuid(uuid!("81494c44-3bed-48e6-b078-1a93a1ae0e29")),
        ObjectCookie(uuid!("a29885a9-0212-4940-964f-e7302131714b")),
    );
    assert_serialize_eq(&value1, serialized);
    assert_deserialize_eq(&value1, serialized);

    let value2 = Value::ObjectId(value1);
    assert_serialize_eq(&value2, serialized);
    assert_deserialize_eq(&value2, serialized);
}

#[test]
fn test_service_id() {
    let serialized = [
        16, 0x81, 0x49, 0x4c, 0x44, 0x3b, 0xed, 0x48, 0xe6, 0xb0, 0x78, 0x1a, 0x93, 0xa1, 0xae,
        0x0e, 0x29, 0xa2, 0x98, 0x85, 0xa9, 0x02, 0x12, 0x49, 0x40, 0x96, 0x4f, 0xe7, 0x30, 0x21,
        0x31, 0x71, 0x4b, 0x04, 0x2e, 0xd5, 0x78, 0x1e, 0x74, 0x43, 0x65, 0x94, 0xb0, 0x3f, 0x76,
        0xfa, 0xcf, 0xb8, 0xb4, 0x73, 0xe7, 0x2e, 0x6b, 0x12, 0xc3, 0x49, 0xfc, 0x9d, 0xfc, 0xe4,
        0xf0, 0xbf, 0x19, 0x17, 0xb1,
    ];

    let value1 = ServiceId::new(
        ObjectId::new(
            ObjectUuid(uuid!("81494c44-3bed-48e6-b078-1a93a1ae0e29")),
            ObjectCookie(uuid!("a29885a9-0212-4940-964f-e7302131714b")),
        ),
        ServiceUuid(uuid!("042ed578-1e74-4365-94b0-3f76facfb8b4")),
        ServiceCookie(uuid!("73e72e6b-12c3-49fc-9dfc-e4f0bf1917b1")),
    );
    assert_serialize_eq(&value1, serialized);
    assert_deserialize_eq(&value1, serialized);

    let value2 = Value::ServiceId(value1);
    assert_serialize_eq(&value2, serialized);
    assert_deserialize_eq(&value2, serialized);
}

#[test]
fn test_object_uuid() {
    let serialized = [
        14, 0x81, 0x49, 0x4c, 0x44, 0x3b, 0xed, 0x48, 0xe6, 0xb0, 0x78, 0x1a, 0x93, 0xa1, 0xae,
        0x0e, 0x29,
    ];

    let value = ObjectUuid(uuid!("81494c44-3bed-48e6-b078-1a93a1ae0e29"));
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
}

#[test]
fn test_object_cookie() {
    let serialized = [
        14, 0x81, 0x49, 0x4c, 0x44, 0x3b, 0xed, 0x48, 0xe6, 0xb0, 0x78, 0x1a, 0x93, 0xa1, 0xae,
        0x0e, 0x29,
    ];

    let value = ObjectCookie(uuid!("81494c44-3bed-48e6-b078-1a93a1ae0e29"));
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
}

#[test]
fn test_service_uuid() {
    let serialized = [
        14, 0x81, 0x49, 0x4c, 0x44, 0x3b, 0xed, 0x48, 0xe6, 0xb0, 0x78, 0x1a, 0x93, 0xa1, 0xae,
        0x0e, 0x29,
    ];

    let value = ServiceUuid(uuid!("81494c44-3bed-48e6-b078-1a93a1ae0e29"));
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
}

#[test]
fn test_service_cookie() {
    let serialized = [
        14, 0x81, 0x49, 0x4c, 0x44, 0x3b, 0xed, 0x48, 0xe6, 0xb0, 0x78, 0x1a, 0x93, 0xa1, 0xae,
        0x0e, 0x29,
    ];

    let value = ServiceCookie(uuid!("81494c44-3bed-48e6-b078-1a93a1ae0e29"));
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
}

#[test]
fn test_channel_cookie() {
    let serialized = [
        14, 0x81, 0x49, 0x4c, 0x44, 0x3b, 0xed, 0x48, 0xe6, 0xb0, 0x78, 0x1a, 0x93, 0xa1, 0xae,
        0x0e, 0x29,
    ];

    let value = ChannelCookie(uuid!("81494c44-3bed-48e6-b078-1a93a1ae0e29"));
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
}

#[test]
fn test_channel_end() {
    let serialized = [40, 0, 0];
    let value = ChannelEnd::Sender;
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);

    let serialized = [40, 0, 0];
    let value = Value::Enum(Box::new(Enum {
        variant: ChannelEnd::Sender as u32,
        value: Value::None,
    }));
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);

    let serialized = [40, 1, 0];
    let value = ChannelEnd::Receiver;
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);

    let serialized = [40, 1, 0];
    let value = Value::Enum(Box::new(Enum {
        variant: ChannelEnd::Receiver as u32,
        value: Value::None,
    }));
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
}

#[test]
fn test_cow() {
    let serialized = [13, 4, b'a', b'b', b'c', b'd'];
    let value = Cow::Borrowed("abcd");
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);

    let serialized = [13, 4, b'a', b'b', b'c', b'd'];
    let value = Cow::<str>::Owned("abcd".to_string());
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
}

#[test]
fn test_cow_bytes() {
    let serialized = [18, 3, 1, 2, 3];
    let value = Cow::Borrowed(ByteSlice::new(&[1, 2, 3]));
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);

    let serialized = [18, 3, 1, 2, 3];
    let value = Cow::<ByteSlice>::Owned(vec![1, 2, 3].into());
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
}
