#[test]
fn test_bytes() {
    let serialized = [18, 3, 1, 2, 3];

    let value = Bytes::new([1, 2, 3]);
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    assert_as_serialize_arg_eq(&value);

    let value = Value::Bytes(Bytes(Vec::from([1, 2, 3])));
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    assert_as_serialize_arg_eq(&value);
}

#[test]
fn test_ext_bytes() {
    let serialized = [18, 3, 1, 2, 3];
    let value = bytes::Bytes::from_iter([1, 2, 3]);
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    assert_as_serialize_arg_eq(&value);
}

#[test]
fn test_ext_bytes_mut() {
    let serialized = [18, 3, 1, 2, 3];
    let value = bytes::BytesMut::from_iter([1, 2, 3]);
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    assert_as_serialize_arg_eq(&value);
}

#[test]
fn test_byte_slice() {
    let serialized = [18, 3, 1, 2, 3];
    let value = ByteSlice::new(&[1, 2, 3]);
    assert_serialize_eq(value, serialized);
    assert_as_serialize_arg_eq_with::<_, Bytes>(value);
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
fn test_u8_hash_map() {
    let serialized1 = [19, 2, 0, 3, 1, 2, 3, 3];
    let serialized2 = [19, 2, 2, 3, 3, 0, 3, 1];

    let value = HashMap::<u8, u8>::from_iter([(0, 1), (2, 3)]);
    let mut buf = bytes::BytesMut::new();
    value
        .serialize(Serializer::new(&mut buf, 0).unwrap())
        .unwrap();
    assert!((buf[..] == serialized1) || (buf[..] == serialized2));
    assert_deserialize_eq(&value, serialized1);
    assert_deserialize_eq(&value, serialized2);
    assert_as_serialize_arg_eq(&value);

    let value = Value::U8Map(HashMap::from_iter([(0, Value::U8(1)), (2, Value::U8(3))]));
    let mut buf = bytes::BytesMut::new();
    value
        .serialize(Serializer::new(&mut buf, 0).unwrap())
        .unwrap();
    assert!((buf[..] == serialized1) || (buf[..] == serialized2));
    assert_deserialize_eq(&value, serialized1);
    assert_deserialize_eq(&value, serialized2);
    assert_as_serialize_arg_eq(&value);
}

#[test]
fn test_u8_btree_map() {
    let serialized = [19, 2, 0, 3, 1, 2, 3, 3];
    let value = BTreeMap::<u8, u8>::from_iter([(0, 1), (2, 3)]);
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    assert_as_serialize_arg_eq(&value);
}

#[test]
fn test_i8_hash_map() {
    let serialized = [20, 1, 2, 3, 4];

    let value = HashMap::<i8, u8>::from_iter([(2, 4)]);
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    assert_as_serialize_arg_eq(&value);

    let value = Value::I8Map(HashMap::from_iter([(2, Value::U8(4))]));
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    assert_as_serialize_arg_eq(&value);
}

#[test]
fn test_i8_btree_map() {
    let serialized = [20, 1, 2, 3, 4];
    let value = BTreeMap::from_iter([(2i8, 4u8)]);
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    assert_as_serialize_arg_eq(&value);
}

#[test]
fn test_u16_hash_map() {
    let serialized = [21, 1, 2, 3, 4];

    let value = HashMap::<u16, u8>::from_iter([(2, 4)]);
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    assert_as_serialize_arg_eq(&value);

    let value = Value::U16Map(HashMap::from_iter([(2, Value::U8(4))]));
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    assert_as_serialize_arg_eq(&value);
}

#[test]
fn test_u16_btree_map() {
    let serialized = [21, 1, 2, 3, 4];
    let value = BTreeMap::from_iter([(2u16, 4u8)]);
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    assert_as_serialize_arg_eq(&value);
}

#[test]
fn test_i16_hash_map() {
    let serialized = [22, 1, 4, 3, 4];

    let value = HashMap::<i16, u8>::from_iter([(2, 4)]);
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    assert_as_serialize_arg_eq(&value);

    let value = Value::I16Map(HashMap::from_iter([(2, Value::U8(4))]));
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    assert_as_serialize_arg_eq(&value);
}

#[test]
fn test_i16_btree_map() {
    let serialized = [22, 1, 4, 3, 4];
    let value = BTreeMap::from_iter([(2i16, 4u8)]);
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    assert_as_serialize_arg_eq(&value);
}

#[test]
fn test_u32_hash_map() {
    let serialized = [23, 1, 2, 3, 4];

    let value = HashMap::<u32, u8>::from_iter([(2, 4)]);
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    assert_as_serialize_arg_eq(&value);

    let value = Value::U32Map(HashMap::from_iter([(2, Value::U8(4))]));
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    assert_as_serialize_arg_eq(&value);
}

#[test]
fn test_u32_btree_map() {
    let serialized = [23, 1, 2, 3, 4];
    let value = BTreeMap::from_iter([(2u32, 4u8)]);
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    assert_as_serialize_arg_eq(&value);
}

#[test]
fn test_i32_hash_map() {
    let serialized = [24, 1, 4, 3, 4];

    let value = HashMap::<i32, u8>::from_iter([(2, 4)]);
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    assert_as_serialize_arg_eq(&value);

    let value = Value::I32Map(HashMap::from_iter([(2, Value::U8(4))]));
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    assert_as_serialize_arg_eq(&value);
}

#[test]
fn test_i32_btree_map() {
    let serialized = [24, 1, 4, 3, 4];
    let value = BTreeMap::from_iter([(2i32, 4u8)]);
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    assert_as_serialize_arg_eq(&value);
}

#[test]
fn test_u64_hash_map() {
    let serialized = [25, 1, 2, 3, 4];

    let value = HashMap::<u64, u8>::from_iter([(2, 4)]);
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    assert_as_serialize_arg_eq(&value);

    let value = Value::U64Map(HashMap::from_iter([(2, Value::U8(4))]));
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    assert_as_serialize_arg_eq(&value);
}

#[test]
fn test_u64_btree_map() {
    let serialized = [25, 1, 2, 3, 4];
    let value = BTreeMap::from_iter([(2u64, 4u8)]);
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    assert_as_serialize_arg_eq(&value);
}

#[test]
fn test_i64_hash_map() {
    let serialized = [26, 1, 4, 3, 4];

    let value = HashMap::<i64, u8>::from_iter([(2, 4)]);
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    assert_as_serialize_arg_eq(&value);

    let value = Value::I64Map(HashMap::from_iter([(2, Value::U8(4))]));
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    assert_as_serialize_arg_eq(&value);
}

#[test]
fn test_i64_btree_map() {
    let serialized = [26, 1, 4, 3, 4];
    let value = BTreeMap::from_iter([(2i64, 4u8)]);
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    assert_as_serialize_arg_eq(&value);
}

#[test]
fn test_string_hash_map() {
    let serialized = [27, 1, 2, b'3', b'4', 5, 6];

    let value = HashMap::<String, u16>::from_iter([("34".to_owned(), 6)]);
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    assert_as_serialize_arg_eq(&value);

    let value = Value::StringMap(HashMap::from_iter([("34".to_owned(), Value::U16(6))]));
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    assert_as_serialize_arg_eq(&value);
}

#[test]
fn test_str_hash_map() {
    let serialized = [27, 1, 2, b'3', b'4', 5, 6];
    let value = HashMap::<&str, u16>::from_iter([("34", 6)]);
    assert_serialize_eq(&value, serialized);
    assert_as_serialize_arg_with::<_, HashMap<String, u16>>(&value);
}

#[test]
fn test_string_btree_map() {
    let serialized = [27, 1, 2, b'3', b'4', 5, 6];
    let value = BTreeMap::from_iter([("34".to_owned(), 6u16)]);
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    assert_as_serialize_arg_eq(&value);
}

#[test]
fn test_str_btree_map() {
    let serialized = [27, 1, 2, b'3', b'4', 5, 6];
    let value = BTreeMap::from_iter([("34", 6u16)]);
    assert_serialize_eq(&value, serialized);
    assert_as_serialize_arg_with::<_, BTreeMap<String, u16>>(&value);
}

#[test]
fn test_uuid_hash_map() {
    let uuid = uuid!("81494c44-3bed-48e6-b078-1a93a1ae0e29");
    let serialized = [
        28, 1, 0x81, 0x49, 0x4c, 0x44, 0x3b, 0xed, 0x48, 0xe6, 0xb0, 0x78, 0x1a, 0x93, 0xa1, 0xae,
        0x0e, 0x29, 3, 0,
    ];

    let value = HashMap::<_, _>::from_iter([(uuid, 0u8)]);
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    assert_as_serialize_arg_eq(&value);

    let value = Value::UuidMap(HashMap::<_, _>::from_iter([(uuid, Value::U8(0))]));
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    assert_as_serialize_arg_eq(&value);
}

#[test]
fn test_uuid_btree_map() {
    let uuid = uuid!("81494c44-3bed-48e6-b078-1a93a1ae0e29");
    let serialized = [
        28, 1, 0x81, 0x49, 0x4c, 0x44, 0x3b, 0xed, 0x48, 0xe6, 0xb0, 0x78, 0x1a, 0x93, 0xa1, 0xae,
        0x0e, 0x29, 3, 0,
    ];

    let value = BTreeMap::from_iter([(uuid, 0u8)]);
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    assert_as_serialize_arg_eq(&value);
}

#[test]
fn test_u8_hash_set() {
    let serialized1 = [29, 2, 3, 4];
    let serialized2 = [29, 2, 4, 3];

    let value = HashSet::<_>::from_iter([3u8, 4]);
    let mut buf = bytes::BytesMut::new();
    value
        .serialize(Serializer::new(&mut buf, 0).unwrap())
        .unwrap();
    assert!((buf[..] == serialized1) || (buf[..] == serialized2));
    assert_deserialize_eq(&value, serialized1);
    assert_deserialize_eq(&value, serialized2);
    assert_as_serialize_arg_eq(&value);

    let value = Value::U8Set(value);
    let mut buf = bytes::BytesMut::new();
    value
        .serialize(Serializer::new(&mut buf, 0).unwrap())
        .unwrap();
    assert!((buf[..] == serialized1) || (buf[..] == serialized2));
    assert_deserialize_eq(&value, serialized1);
    assert_deserialize_eq(&value, serialized2);
    assert_as_serialize_arg_eq(&value);
}

#[test]
fn test_u8_btree_set() {
    let serialized = [29, 2, 3, 4];
    let value = BTreeSet::from_iter([4u8, 3]);
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    assert_as_serialize_arg_eq(&value);
}

#[test]
fn test_i8_hash_set() {
    let serialized = [30, 1, 2];

    let value = HashSet::<_>::from_iter([2i8]);
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    assert_as_serialize_arg_eq(&value);

    let value = Value::I8Set(HashSet::from_iter([2]));
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    assert_as_serialize_arg_eq(&value);
}

#[test]
fn test_i8_btree_set() {
    let serialized = [30, 1, 2];
    let value = BTreeSet::from_iter([2i8]);
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    assert_as_serialize_arg_eq(&value);
}

#[test]
fn test_u16_hash_set() {
    let serialized = [31, 1, 2];

    let value = HashSet::<_>::from_iter([2u16]);
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    assert_as_serialize_arg_eq(&value);

    let value = Value::U16Set(HashSet::from_iter([2]));
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    assert_as_serialize_arg_eq(&value);
}

#[test]
fn test_u16_btree_set() {
    let serialized = [31, 1, 2];
    let value = BTreeSet::from_iter([2u16]);
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    assert_as_serialize_arg_eq(&value);
}

#[test]
fn test_i16_hash_set() {
    let serialized = [32, 1, 2];

    let value = HashSet::<_>::from_iter([1i16]);
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    assert_as_serialize_arg_eq(&value);

    let value = Value::I16Set(HashSet::from_iter([1]));
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    assert_as_serialize_arg_eq(&value);
}

#[test]
fn test_i16_btree_set() {
    let serialized = [32, 1, 2];
    let value = BTreeSet::from_iter([1i16]);
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    assert_as_serialize_arg_eq(&value);
}

#[test]
fn test_u32_hash_set() {
    let serialized = [33, 1, 2];

    let value = HashSet::<_>::from_iter([2u32]);
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    assert_as_serialize_arg_eq(&value);

    let value = Value::U32Set(HashSet::from_iter([2]));
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    assert_as_serialize_arg_eq(&value);
}

#[test]
fn test_u32_btree_set() {
    let serialized = [33, 1, 2];
    let value = BTreeSet::from_iter([2u32]);
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    assert_as_serialize_arg_eq(&value);
}

#[test]
fn test_i32_hash_set() {
    let serialized = [34, 1, 2];

    let value = HashSet::<_>::from_iter([1i32]);
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    assert_as_serialize_arg_eq(&value);

    let value = Value::I32Set(HashSet::from_iter([1]));
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    assert_as_serialize_arg_eq(&value);
}

#[test]
fn test_i32_btree_set() {
    let serialized = [34, 1, 2];
    let value = BTreeSet::from_iter([1i32]);
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    assert_as_serialize_arg_eq(&value);
}

#[test]
fn test_u64_hash_set() {
    let serialized = [35, 1, 2];

    let value = HashSet::<_>::from_iter([2u64]);
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    assert_as_serialize_arg_eq(&value);

    let value = Value::U64Set(HashSet::from_iter([2]));
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    assert_as_serialize_arg_eq(&value);
}

#[test]
fn test_u64_btree_set() {
    let serialized = [35, 1, 2];
    let value = BTreeSet::from_iter([2u64]);
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    assert_as_serialize_arg_eq(&value);
}

#[test]
fn test_i64_hash_set() {
    let serialized = [36, 1, 2];

    let value = HashSet::<_>::from_iter([1i64]);
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    assert_as_serialize_arg_eq(&value);

    let value = Value::I64Set(HashSet::from_iter([1]));
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    assert_as_serialize_arg_eq(&value);
}

#[test]
fn test_i64_btree_set() {
    let serialized = [36, 1, 2];
    let value = BTreeSet::from_iter([1i64]);
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    assert_as_serialize_arg_eq(&value);
}

#[test]
fn test_string_hash_set() {
    let serialized = [37, 1, 2, b'3', b'4'];

    let value = HashSet::<_>::from_iter(["34".to_owned()]);
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    assert_as_serialize_arg_eq(&value);

    let value = Value::StringSet(HashSet::from_iter(["34".to_owned()]));
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    assert_as_serialize_arg_eq(&value);
}

#[test]
fn test_str_hash_set() {
    let serialized = [37, 1, 2, b'3', b'4'];
    let value = HashSet::<_>::from_iter(["34"]);
    assert_serialize_eq(&value, serialized);
    assert_as_serialize_arg_with::<_, HashSet<String>>(&value);
}

#[test]
fn test_string_btree_set() {
    let serialized = [37, 1, 2, b'3', b'4'];
    let value = BTreeSet::from_iter(["34".to_owned()]);
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    assert_as_serialize_arg_eq(&value);
}

#[test]
fn test_str_btree_set() {
    let serialized = [37, 1, 2, b'3', b'4'];
    let value = BTreeSet::from_iter(["34"]);
    assert_serialize_eq(&value, serialized);
    assert_as_serialize_arg_with::<_, BTreeSet<String>>(&value);
}

#[test]
fn test_uuid_hash_set() {
    let uuid = uuid!("81494c44-3bed-48e6-b078-1a93a1ae0e29");
    let serialized = [
        38, 1, 0x81, 0x49, 0x4c, 0x44, 0x3b, 0xed, 0x48, 0xe6, 0xb0, 0x78, 0x1a, 0x93, 0xa1, 0xae,
        0x0e, 0x29,
    ];

    let value = HashSet::<_>::from_iter([uuid]);
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    assert_as_serialize_arg_eq(&value);

    let value = Value::UuidSet(HashSet::<_>::from_iter([uuid]));
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    assert_as_serialize_arg_eq(&value);
}

#[test]
fn test_uuid_btree_set() {
    let uuid = uuid!("81494c44-3bed-48e6-b078-1a93a1ae0e29");
    let serialized = [
        38, 1, 0x81, 0x49, 0x4c, 0x44, 0x3b, 0xed, 0x48, 0xe6, 0xb0, 0x78, 0x1a, 0x93, 0xa1, 0xae,
        0x0e, 0x29,
    ];

    let value = BTreeSet::from_iter([uuid]);
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    assert_as_serialize_arg_eq(&value);
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
            .serialize_field(0u32, &self.a)?
            .serialize_field(1u32, &self.b)?;
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

        deserializer.finish_with(|_| {
            Ok(Self {
                a: a.ok_or(DeserializeError::InvalidSerialization)?,
                b,
            })
        })
    }
}

#[test]
fn test_struct_1() {
    let serialized = [39, 2, 0, 3, 4, 1, 0];
    let value = TestStruct { a: 4, b: None };
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
}

#[test]
fn test_struct_2() {
    let serialized = [39, 2, 0, 3, 4, 1, 1, 13, 1, b'a'];
    let value = TestStruct {
        a: 4,
        b: Some("a".to_owned()),
    };
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
}

#[test]
fn test_struct_3() {
    let serialized = [39, 1, 0, 3, 4];
    let value = Value::Struct(Struct(HashMap::from_iter([(0, Value::U8(4))])));
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    assert_as_serialize_arg_eq(&value);
}

#[test]
fn test_struct_4() {
    let serialized = [39, 1, 0, 1, 3, 4];
    let value = Value::Struct(Struct(HashMap::from_iter([(
        0,
        Value::Some(Box::new(Value::U8(4))),
    )])));
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    assert_as_serialize_arg_eq(&value);
}

#[test]
fn test_struct_5() {
    let serialized = [39, 1, 0, 0];
    let value = Value::Struct(Struct(HashMap::from_iter([(0, Value::None)])));
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    assert_as_serialize_arg_eq(&value);
}

#[derive(Debug, PartialEq, Eq)]
enum TestEnum {
    A(u8),
    B(Option<String>),
}

impl Serialize for TestEnum {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        match self {
            Self::A(value) => serializer.serialize_enum(0u32, value),
            Self::B(value) => serializer.serialize_enum(1u32, value),
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
fn test_enum_1() {
    let serialized = [40, 0, 3, 4];

    let value = TestEnum::A(4);
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);

    let value = Value::Enum(Box::new(Enum::new(0, Value::U8(4))));
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    assert_as_serialize_arg_eq(&value);
}

#[test]
fn test_enum_2() {
    let serialized = [40, 1, 0];

    let value = TestEnum::B(None);
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);

    let value = Value::Enum(Box::new(Enum::new(1, Value::None)));
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    assert_as_serialize_arg_eq(&value);
}

#[test]
fn test_enum_3() {
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
    assert_as_serialize_arg_eq(&value);
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
    assert_as_serialize_arg_eq(&value);
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
    assert_as_serialize_arg_eq(&value);
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
    assert_as_serialize_arg_eq(&value);
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
    assert_as_serialize_arg_eq(&value);
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
    assert_as_serialize_arg_eq(&value);
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
    assert_as_serialize_arg_eq(&value);
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
    assert_as_serialize_arg_eq(&value);
}

#[test]
fn test_cow() {
    let serialized = [13, 4, b'a', b'b', b'c', b'd'];
    let value = Cow::Borrowed("abcd");
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    assert_as_serialize_arg_eq(&value);

    let serialized = [13, 4, b'a', b'b', b'c', b'd'];
    let value = Cow::<str>::Owned("abcd".to_string());
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    assert_as_serialize_arg_eq(&value);
}

#[test]
fn test_cow_bytes_owned() {
    let serialized = [18, 3, 1, 2, 3];
    let value = Cow::<ByteSlice>::Owned(vec![1, 2, 3].into());
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    assert_as_serialize_arg_eq(&value);
}

#[test]
fn test_cow_bytes_borrowed() {
    let serialized = [18, 3, 1, 2, 3];
    let value = Cow::Borrowed(ByteSlice::new(&[1, 2, 3]));
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    assert_as_serialize_arg_eq(&value);
}

#[test]
fn test_serialize_too_deep() {
    let value = Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(
        Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(
            Some(Some(Some(Some(Some(Some(Some(Some(Some(Some(0)))))))))),
        ))))))))))),
    )))))))))));

    assert_eq!(
        SerializedValue::serialize(&value),
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
fn test_result_ok() {
    let serialized = [40, 0, 3, 1];
    let value = Result::<u8, u8>::Ok(1);
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    let _ = assert_as_serialize_arg_eq(&value);
}

#[test]
fn test_result_err() {
    let serialized = [40, 1, 3, 1];
    let value = Result::<u8, u8>::Err(1);
    assert_serialize_eq(&value, serialized);
    assert_deserialize_eq(&value, serialized);
    let _ = assert_as_serialize_arg_eq(&value);
}

#[test]
fn test_infallible() {
    let serialized = SerializedValue::serialize(&()).unwrap();
    let res = serialized.deserialize::<Infallible>();
    assert_eq!(res, Err(DeserializeError::UnexpectedValue));
}
