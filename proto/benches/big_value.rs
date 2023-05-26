use aldrin_proto::{
    Bytes, ChannelCookie, Deserialize, DeserializeError, Deserializer, ObjectCookie, ObjectId,
    ObjectUuid, Serialize, SerializeError, SerializedValue, Serializer, ServiceCookie, ServiceId,
    ServiceUuid,
};
use criterion::{black_box, Criterion};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

pub fn serialize(c: &mut Criterion) {
    let big_value = big_value();

    c.bench_function("big-value/serialize", |b| {
        b.iter(|| SerializedValue::serialize(black_box(&big_value)))
    });
}

pub fn deserialize(c: &mut Criterion) {
    let big_value = SerializedValue::serialize(&big_value()).unwrap();

    c.bench_function("big-value/deserialize", |b| {
        b.iter(|| black_box(&big_value).deserialize::<BigValue>())
    });
}

fn big_value() -> BigValue {
    const UUID1: Uuid = uuid::uuid!("00000000-0000-0000-0000-000000000001");
    const UUID2: Uuid = uuid::uuid!("00000000-0000-0000-0000-000000000002");
    const UUID3: Uuid = uuid::uuid!("00000000-0000-0000-0000-000000000003");
    const UUID4: Uuid = uuid::uuid!("00000000-0000-0000-0000-000000000004");

    BigValue {
        none: None,
        some: Some(()),
        bool_: true,
        u8_: 0,
        i8_: 1,
        u16_: 2,
        i16_: 3,
        u32_: 4,
        i32_: 5,
        u64_: 6,
        i64_: 7,
        f32_: 8.0,
        f64_: 9.0,
        string: "big value".to_owned(),
        uuid: UUID1,
        object_id: ObjectId {
            uuid: ObjectUuid(UUID1),
            cookie: ObjectCookie(UUID2),
        },
        service_id: ServiceId {
            object_id: ObjectId {
                uuid: ObjectUuid(UUID1),
                cookie: ObjectCookie(UUID2),
            },
            uuid: ServiceUuid(UUID3),
            cookie: ServiceCookie(UUID4),
        },
        vec: vec![0, 1, 2, 3],
        bytes: Bytes(vec![0, 1, 2, 3]),
        u8_map: [(0, 0), (1, 1), (2, 2), (3, 3)].into_iter().collect(),
        i8_map: [(0, 0), (1, 1), (2, 2), (3, 3)].into_iter().collect(),
        u16_map: [(0, 0), (1, 1), (2, 2), (3, 3)].into_iter().collect(),
        i16_map: [(0, 0), (1, 1), (2, 2), (3, 3)].into_iter().collect(),
        u32_map: [(0, 0), (1, 1), (2, 2), (3, 3)].into_iter().collect(),
        i32_map: [(0, 0), (1, 1), (2, 2), (3, 3)].into_iter().collect(),
        u64_map: [(0, 0), (1, 1), (2, 2), (3, 3)].into_iter().collect(),
        i64_map: [(0, 0), (1, 1), (2, 2), (3, 3)].into_iter().collect(),
        string_map: [
            ("big".to_owned(), 0),
            ("value".to_owned(), 1),
            ("foo".to_owned(), 2),
            ("bar".to_owned(), 3),
        ]
        .into_iter()
        .collect(),
        uuid_map: [(UUID1, 0), (UUID2, 1), (UUID3, 2), (UUID4, 3)]
            .into_iter()
            .collect(),
        u8_set: [0, 1, 2, 3].into_iter().collect(),
        i8_set: [0, 1, 2, 3].into_iter().collect(),
        u16_set: [0, 1, 2, 3].into_iter().collect(),
        i16_set: [0, 1, 2, 3].into_iter().collect(),
        u32_set: [0, 1, 2, 3].into_iter().collect(),
        i32_set: [0, 1, 2, 3].into_iter().collect(),
        u64_set: [0, 1, 2, 3].into_iter().collect(),
        i64_set: [0, 1, 2, 3].into_iter().collect(),
        string_set: [
            "big".to_owned(),
            "value".to_owned(),
            "foo".to_owned(),
            "bar".to_owned(),
        ]
        .into_iter()
        .collect(),
        uuid_set: [UUID1, UUID2, UUID3, UUID4].into_iter().collect(),
        struct_: SmallStruct {
            none: None,
            some: Some(()),
            bool_: true,
            u8_: 0,
        },
        enum_: SmallEnum::Variant(SmallStruct {
            none: None,
            some: Some(()),
            bool_: true,
            u8_: 0,
        }),
        sender: Sender(ChannelCookie(UUID1)),
        receiver: Receiver(ChannelCookie(UUID1)),
    }
}

#[derive(aldrin_proto_macros::Serialize, aldrin_proto_macros::Deserialize)]
struct BigValue {
    none: Option<()>,
    some: Option<()>,
    bool_: bool,
    u8_: u8,
    i8_: i8,
    u16_: u16,
    i16_: i16,
    u32_: u32,
    i32_: i32,
    u64_: u64,
    i64_: i64,
    f32_: f32,
    f64_: f64,
    string: String,
    uuid: Uuid,
    object_id: ObjectId,
    service_id: ServiceId,
    vec: Vec<i32>,
    bytes: Bytes,
    u8_map: HashMap<u8, i32>,
    i8_map: HashMap<i8, i32>,
    u16_map: HashMap<u16, i32>,
    i16_map: HashMap<i16, i32>,
    u32_map: HashMap<u32, i32>,
    i32_map: HashMap<i32, i32>,
    u64_map: HashMap<u64, i32>,
    i64_map: HashMap<i64, i32>,
    string_map: HashMap<String, i32>,
    uuid_map: HashMap<Uuid, i32>,
    u8_set: HashSet<u8>,
    i8_set: HashSet<u8>,
    u16_set: HashSet<u16>,
    i16_set: HashSet<u16>,
    u32_set: HashSet<u32>,
    i32_set: HashSet<u32>,
    u64_set: HashSet<u64>,
    i64_set: HashSet<u64>,
    string_set: HashSet<String>,
    uuid_set: HashSet<Uuid>,
    struct_: SmallStruct,
    enum_: SmallEnum,
    sender: Sender,
    receiver: Receiver,
}

#[derive(aldrin_proto_macros::Serialize, aldrin_proto_macros::Deserialize)]
struct SmallStruct {
    none: Option<()>,
    some: Option<()>,
    bool_: bool,
    u8_: u8,
}

#[derive(aldrin_proto_macros::Serialize, aldrin_proto_macros::Deserialize)]
enum SmallEnum {
    Variant(SmallStruct),
}

struct Sender(ChannelCookie);

impl Serialize for Sender {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_sender(self.0);
        Ok(())
    }
}

impl Deserialize for Sender {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_sender().map(Self)
    }
}

struct Receiver(ChannelCookie);

impl Serialize for Receiver {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_receiver(self.0);
        Ok(())
    }
}

impl Deserialize for Receiver {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_receiver().map(Self)
    }
}
