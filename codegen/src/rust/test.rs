use aldrin::core::tags::{self, PrimaryTag};
use aldrin::core::{
    Deserialize, DeserializeError, Deserializer, ObjectUuid, Serialize, SerializeError,
    SerializedValue, Serializer,
};
use aldrin::low_level::Proxy;
use aldrin::Error;
use aldrin_test::tokio::TestBroker;
use futures_util::stream::StreamExt;
use subscribe_all::SubscribeAllEvent;
use uuid::uuid;

aldrin::generate!("test/all_types.aldrin");
aldrin::generate!("test/before_derive_compat.aldrin");
aldrin::generate!("test/constants.aldrin");
aldrin::generate!("test/enum_fallback.aldrin");
aldrin::generate!("test/extern.aldrin", introspection = true);
aldrin::generate!("test/generic_struct.aldrin");
aldrin::generate!("test/introspection.aldrin", introspection = true);
aldrin::generate!("test/old_new.aldrin");
aldrin::generate!("test/options.aldrin");
aldrin::generate!("test/result.aldrin");
aldrin::generate!("test/subscribe_all.aldrin");
aldrin::generate!("test/test1.aldrin");
aldrin::generate!("test/unit.aldrin");

aldrin::generate!(
    "test/raw_identifiers.aldrin",
    include = "test",
    introspection = true
);

mod conditional_introspection {
    mod available {
        aldrin::generate!("test/introspection.aldrin", introspection_if = "rust");
    }

    #[allow(unexpected_cfgs)]
    mod unavailable {
        aldrin::generate!("test/introspection.aldrin", introspection_if = "disabled");
    }
}

mod empty_introspection {
    aldrin::generate!(
        "test/introspection.aldrin",
        introspection = true,
        client = false,
        server = false,
    );
}

#[tokio::test]
async fn auto_reply_with_invalid_args() {
    let mut broker = TestBroker::new();
    let client = broker.add_client().await;

    let obj = client.create_object(ObjectUuid::new_v4()).await.unwrap();
    let mut svc = test1::Test1::new(&obj).await.unwrap();
    let proxy = Proxy::new(&client, svc.id()).await.unwrap();
    tokio::spawn(async move { while svc.next().await.is_some() {} });

    let err = proxy.call(1, 0, None).await.unwrap_err();
    assert_eq!(err, Error::invalid_arguments(1, None));

    let err = proxy.call(2, (), None).await.unwrap_err();
    assert_eq!(err, Error::invalid_arguments(2, None));
}

#[tokio::test]
async fn auto_reply_with_invalid_function() {
    let mut broker = TestBroker::new();
    let client = broker.add_client().await;

    let obj = client.create_object(ObjectUuid::new_v4()).await.unwrap();
    let mut svc = test1::Test1::new(&obj).await.unwrap();
    let proxy = Proxy::new(&client, svc.id()).await.unwrap();
    tokio::spawn(async move { while svc.next().await.is_some() {} });

    let err = proxy.call(3, (), None).await.unwrap_err();
    assert_eq!(err, Error::invalid_function(3));
}

#[test]
fn constants() {
    assert_eq!(constants::CONST_U8, 1);
    assert_eq!(constants::CONST_I8, 2);
    assert_eq!(constants::CONST_U16, 3);
    assert_eq!(constants::CONST_I16, 4);
    assert_eq!(constants::CONST_U32, 5);
    assert_eq!(constants::CONST_I32, 6);
    assert_eq!(constants::CONST_U64, 7);
    assert_eq!(constants::CONST_I64, 8);
    assert_eq!(constants::CONST_STRING, "string");
    assert_eq!(
        constants::CONST_UUID,
        uuid!("5c368dc9-e6d3-4545-86d1-435fe3e771cc")
    );
}

#[test]
fn generic_struct() {
    let s1 = generic_struct::Struct {
        field1: 1,
        field2: None,
    };

    let s1_serialized = SerializedValue::serialize(&s1).unwrap();
    let g: aldrin::core::Struct = s1_serialized.deserialize().unwrap();
    let g_serialized = SerializedValue::serialize(&g).unwrap();
    let s2: generic_struct::Struct = g_serialized.deserialize().unwrap();

    assert_eq!(s1, s2);
}

#[test]
fn old_as_new() {
    let old = old_new::Old { f1: 1 };
    let old_serialized = SerializedValue::serialize(&old).unwrap();
    let new: old_new::New = old_serialized.deserialize().unwrap();
    assert_eq!(new.f1, 1);
    assert_eq!(new.f2, None);
}

#[test]
fn new_as_old() {
    let new = old_new::New { f1: 1, f2: None };
    let new_serialized = SerializedValue::serialize(&new).unwrap();
    let old: old_new::Old = new_serialized.deserialize().unwrap();
    assert_eq!(old.f1, 1);

    let new = old_new::New { f1: 1, f2: Some(2) };
    let new_serialized = SerializedValue::serialize(&new).unwrap();
    let old: old_new::Old = new_serialized.deserialize().unwrap();
    assert_eq!(old.f1, 1);
}

#[tokio::test]
async fn unsubscribe_all() {
    let mut broker = TestBroker::new();
    let client = broker.add_client().await;

    let obj = client.create_object(ObjectUuid::new_v4()).await.unwrap();
    let svc = subscribe_all::SubscribeAll::new(&obj).await.unwrap();
    let mut proxy = subscribe_all::SubscribeAllProxy::new(&client, svc.id())
        .await
        .unwrap();

    proxy.subscribe_ev1().await.unwrap();
    proxy.inner().subscribe_all().await.unwrap();

    svc.ev1().unwrap();
    svc.ev2().unwrap();

    assert!(matches!(
        proxy.next_event().await.unwrap().unwrap(),
        SubscribeAllEvent::Ev1(_)
    ));

    assert!(matches!(
        proxy.next_event().await.unwrap().unwrap(),
        SubscribeAllEvent::Ev2(_)
    ));

    proxy.unsubscribe_all().await.unwrap();

    svc.ev1().unwrap();
    svc.ev2().unwrap();
    svc.destroy().await.unwrap();

    assert!(proxy.next_event().await.is_none());
}

#[tokio::test]
async fn before_derive_compat_struct() {
    use before_derive_compat::NewStruct;

    #[derive(Debug)]
    pub struct OldStruct {
        pub f1: i32,
        pub f2: Option<i32>,
        pub f3: Option<i32>,
    }

    impl PrimaryTag for OldStruct {
        type Tag = tags::Value;
    }

    impl Serialize<tags::Value> for &OldStruct {
        fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
            let mut serializer = serializer.serialize_struct1(3)?;

            serializer.serialize::<tags::I32, _>(1u32, &self.f1)?;
            serializer.serialize::<tags::Option<tags::I32>, _>(2u32, &self.f2)?;
            serializer.serialize::<tags::Option<tags::I32>, _>(3u32, &self.f3)?;

            serializer.finish()
        }
    }

    impl Deserialize<tags::Value> for OldStruct {
        fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
            let mut deserializer = deserializer.deserialize_struct()?;

            let mut f1 = None;
            let mut f2 = None;
            let mut f3 = None;

            while let Some(deserializer) = deserializer.deserialize()? {
                match deserializer.id() {
                    1 => f1 = deserializer.deserialize::<tags::I32, _>().map(Some)?,

                    2 => {
                        f2 = deserializer
                            .deserialize::<tags::Option<tags::I32>, _>()
                            .map(Some)?
                    }

                    3 => f3 = deserializer.deserialize::<tags::Option<tags::I32>, _>()?,
                    _ => deserializer.skip()?,
                }
            }

            deserializer.finish_with(|_| {
                Ok(Self {
                    f1: f1.ok_or(DeserializeError::InvalidSerialization)?,
                    f2: f2.ok_or(DeserializeError::InvalidSerialization)?,
                    f3,
                })
            })
        }
    }

    let old = OldStruct {
        f1: 0,
        f2: None,
        f3: None,
    };
    let serialized = SerializedValue::serialize(&old).unwrap();
    let new = serialized.deserialize::<NewStruct>().unwrap();
    assert_eq!(new.f1, old.f1);
    assert_eq!(new.f2, old.f2);
    assert_eq!(new.f3, old.f3);

    let old = OldStruct {
        f1: 0,
        f2: Some(1),
        f3: None,
    };
    let serialized = SerializedValue::serialize(&old).unwrap();
    let new = serialized.deserialize::<NewStruct>().unwrap();
    assert_eq!(new.f1, old.f1);
    assert_eq!(new.f2, old.f2);
    assert_eq!(new.f3, old.f3);

    let old = OldStruct {
        f1: 0,
        f2: None,
        f3: Some(2),
    };
    let serialized = SerializedValue::serialize(&old).unwrap();
    let new = serialized.deserialize::<NewStruct>().unwrap();
    assert_eq!(new.f1, old.f1);
    assert_eq!(new.f2, old.f2);
    assert_eq!(new.f3, old.f3);

    let old = OldStruct {
        f1: 0,
        f2: Some(1),
        f3: Some(2),
    };
    let serialized = SerializedValue::serialize(&old).unwrap();
    let new = serialized.deserialize::<NewStruct>().unwrap();
    assert_eq!(new.f1, old.f1);
    assert_eq!(new.f2, old.f2);
    assert_eq!(new.f3, old.f3);

    let new = NewStruct {
        f1: 0,
        f2: None,
        f3: None,
    };
    let serialized = SerializedValue::serialize(&new).unwrap();
    let old = serialized.deserialize::<OldStruct>().unwrap();
    assert_eq!(old.f1, new.f1);
    assert_eq!(old.f2, new.f2);
    assert_eq!(old.f3, new.f3);

    let new = NewStruct {
        f1: 0,
        f2: Some(1),
        f3: None,
    };
    let serialized = SerializedValue::serialize(&new).unwrap();
    let old = serialized.deserialize::<OldStruct>().unwrap();
    assert_eq!(old.f1, new.f1);
    assert_eq!(old.f2, new.f2);
    assert_eq!(old.f3, new.f3);

    let new = NewStruct {
        f1: 0,
        f2: None,
        f3: Some(2),
    };
    let serialized = SerializedValue::serialize(&new).unwrap();
    let old = serialized.deserialize::<OldStruct>().unwrap();
    assert_eq!(old.f1, new.f1);
    assert_eq!(old.f2, new.f2);
    assert_eq!(old.f3, new.f3);

    let new = NewStruct {
        f1: 0,
        f2: Some(1),
        f3: Some(2),
    };
    let serialized = SerializedValue::serialize(&new).unwrap();
    let old = serialized.deserialize::<OldStruct>().unwrap();
    assert_eq!(old.f1, new.f1);
    assert_eq!(old.f2, new.f2);
    assert_eq!(old.f3, new.f3);
}

#[tokio::test]
async fn before_derive_compat_enum() {
    use before_derive_compat::NewEnum;

    #[derive(Debug)]
    pub enum OldEnum {
        Var1,
        Var2(i32),
        Var3(Option<i32>),
    }

    impl PrimaryTag for OldEnum {
        type Tag = tags::Value;
    }

    impl Serialize<tags::Value> for &OldEnum {
        fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
            match self {
                OldEnum::Var1 => serializer.serialize_enum::<tags::Unit, _>(1u32, &()),
                OldEnum::Var2(v) => serializer.serialize_enum::<tags::I32, _>(2u32, v),

                OldEnum::Var3(v) => {
                    serializer.serialize_enum::<tags::Option<tags::I32>, _>(3u32, v)
                }
            }
        }
    }

    impl Deserialize<tags::Value> for OldEnum {
        fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
            let deserializer = deserializer.deserialize_enum()?;

            match deserializer.variant() {
                1 => deserializer
                    .deserialize::<tags::Unit, _>()
                    .map(|()| Self::Var1),

                2 => deserializer.deserialize::<tags::I32, _>().map(Self::Var2),

                3 => deserializer
                    .deserialize::<tags::Option<tags::I32>, _>()
                    .map(Self::Var3),

                _ => Err(DeserializeError::InvalidSerialization),
            }
        }
    }

    impl PartialEq<OldEnum> for NewEnum {
        fn eq(&self, other: &OldEnum) -> bool {
            match (self, other) {
                (Self::Var1, OldEnum::Var1) => true,
                (Self::Var2(v1), OldEnum::Var2(v2)) => v1 == v2,
                (Self::Var3(v1), OldEnum::Var3(v2)) => v1 == v2,
                _ => false,
            }
        }
    }

    let old = OldEnum::Var1;
    let serialized = SerializedValue::serialize(&old).unwrap();
    let new = serialized.deserialize::<NewEnum>().unwrap();
    assert_eq!(new, old);

    let old = OldEnum::Var2(0);
    let serialized = SerializedValue::serialize(&old).unwrap();
    let new = serialized.deserialize::<NewEnum>().unwrap();
    assert_eq!(new, old);

    let old = OldEnum::Var3(None);
    let serialized = SerializedValue::serialize(&old).unwrap();
    let new = serialized.deserialize::<NewEnum>().unwrap();
    assert_eq!(new, old);

    let old = OldEnum::Var3(Some(1));
    let serialized = SerializedValue::serialize(&old).unwrap();
    let new = serialized.deserialize::<NewEnum>().unwrap();
    assert_eq!(new, old);

    let new = NewEnum::Var1;
    let serialized = SerializedValue::serialize(&new).unwrap();
    let old = serialized.deserialize::<OldEnum>().unwrap();
    assert_eq!(new, old);

    let new = NewEnum::Var2(0);
    let serialized = SerializedValue::serialize(&new).unwrap();
    let old = serialized.deserialize::<OldEnum>().unwrap();
    assert_eq!(new, old);

    let new = NewEnum::Var3(None);
    let serialized = SerializedValue::serialize(&new).unwrap();
    let old = serialized.deserialize::<OldEnum>().unwrap();
    assert_eq!(new, old);

    let new = NewEnum::Var3(Some(1));
    let serialized = SerializedValue::serialize(&new).unwrap();
    let old = serialized.deserialize::<OldEnum>().unwrap();
    assert_eq!(new, old);
}

#[test]
fn enum_fallback_new_to_old() {
    use enum_fallback::{New, Old};

    let new = New::Var2(1);
    let serialized = SerializedValue::serialize(&new).unwrap();

    let old = serialized.deserialize::<Old>().unwrap();
    let Old::Unknown(variant) = old else { panic!() };

    assert_eq!(variant.id(), 2);
    assert_eq!(variant.deserialize(), Ok(1u32));

    let serialized = SerializedValue::serialize(Old::Unknown(variant)).unwrap();
    let new2 = serialized.deserialize().unwrap();
    assert_eq!(new, new2);
}
