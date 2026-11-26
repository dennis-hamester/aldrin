use super::{SerializedValue, SerializedValueSlice};
use crate::tags::{PrimaryTag, Tag};
use crate::{
    Deserialize, DeserializeError, Deserializer, MAX_VALUE_DEPTH, Serialize, SerializeError,
    Serializer, ValueKind,
};
use std::collections::HashMap;

#[test]
fn empty() {
    let empty = SerializedValue::empty();
    let unit = SerializedValue::serialize(()).unwrap();

    assert_eq!(empty, unit);
    assert_eq!(empty.kind(), Ok(ValueKind::None));
    assert_eq!(empty.deserialize::<()>(), Ok(()));
    assert_eq!(
        empty.deserialize::<u32>(),
        Err(DeserializeError::UnexpectedValue)
    );
}

#[test]
fn concrete_vs_vague() {
    #[derive(Debug, PartialEq, Eq)]
    struct Type<T1, T2> {
        field1: T1,
        field2: T2,
    }

    impl<T1: Tag, T2: Tag> Tag for Type<T1, T2> {}

    impl<T1: PrimaryTag, T2: PrimaryTag> PrimaryTag for Type<T1, T2> {
        type Tag = Type<T1::Tag, T2::Tag>;
    }

    impl<T1, U1, T2, U2> Serialize<Type<T1, T2>> for Type<U1, U2>
    where
        T1: Tag,
        U1: Serialize<T1>,
        T2: Tag,
        U2: Serialize<T2>,
    {
        fn serialize(self, serializer: Serializer<'_>) -> Result<(), SerializeError> {
            let mut serializer = serializer.serialize_struct2()?;

            serializer.serialize(1u32, self.field1)?;
            serializer.serialize(2u32, self.field2)?;

            serializer.finish()
        }
    }

    impl<'a, T1, U1, T2, U2> Serialize<Type<T1, T2>> for &'a Type<U1, U2>
    where
        T1: Tag,
        &'a U1: Serialize<T1>,
        T2: Tag,
        &'a U2: Serialize<T2>,
    {
        fn serialize(self, serializer: Serializer<'_>) -> Result<(), SerializeError> {
            let mut serializer = serializer.serialize_struct2()?;

            serializer.serialize(1u32, &self.field1)?;
            serializer.serialize(2u32, &self.field2)?;

            serializer.finish()
        }
    }

    impl<T1, U1, T2, U2> Deserialize<Type<T1, T2>> for Type<U1, U2>
    where
        T1: Tag,
        U1: Deserialize<T1>,
        T2: Tag,
        U2: Deserialize<T2>,
    {
        fn deserialize(deserializer: Deserializer<'_, '_>) -> Result<Self, DeserializeError> {
            let mut deserializer = deserializer.deserialize_struct()?;

            let mut field1 = None;
            let mut field2 = None;

            while let Some(deserializer) = deserializer.deserialize()? {
                match deserializer.id() {
                    1 => field1 = deserializer.deserialize().map(Some)?,
                    2 => field2 = deserializer.deserialize().map(Some)?,
                    _ => deserializer.skip()?,
                }
            }

            deserializer.finish_with(|_| {
                Ok(Self {
                    field1: field1.ok_or(DeserializeError::InvalidSerialization)?,
                    field2: field2.ok_or(DeserializeError::InvalidSerialization)?,
                })
            })
        }
    }

    type Concrete = Type<i32, String>;
    type Vague = Type<SerializedValue, SerializedValue>;

    let concrete = Concrete {
        field1: 0x1234,
        field2: "0x1234".to_owned(),
    };

    let vague = Vague {
        field1: SerializedValue::serialize(0x1234).unwrap(),
        field2: SerializedValue::serialize("0x1234").unwrap(),
    };

    assert_eq!(
        concrete,
        SerializedValue::serialize(&vague)
            .unwrap()
            .deserialize()
            .unwrap()
    );

    assert_eq!(
        vague,
        SerializedValue::serialize(&concrete)
            .unwrap()
            .deserialize()
            .unwrap()
    );
}

#[test]
fn validate() {
    assert_eq!(SerializedValue::empty().validate(), Ok(()));
    assert_eq!(
        SerializedValue::serialize(42u32).unwrap().validate(),
        Ok(())
    );
    assert_eq!(
        SerializedValue::serialize(vec![1u32, 2, 3])
            .unwrap()
            .validate(),
        Ok(())
    );
    assert_eq!(
        SerializedValue::serialize(HashMap::<u32, String>::from_iter([(
            u32::MAX,
            "hello".to_owned(),
        )]))
        .unwrap()
        .validate(),
        Ok(())
    );

    assert_eq!(
        SerializedValueSlice::new(&[]).validate(),
        Err(DeserializeError::UnexpectedEoi)
    );
    assert_eq!(
        SerializedValueSlice::new(&[255]).validate(),
        Err(DeserializeError::InvalidSerialization)
    );
    assert_eq!(
        SerializedValueSlice::new(&[ValueKind::U32.into(), 255, 1]).validate(),
        Err(DeserializeError::UnexpectedEoi)
    );
    assert_eq!(
        SerializedValueSlice::new(&[ValueKind::String.into(), 5, b'a']).validate(),
        Err(DeserializeError::UnexpectedEoi)
    );
    assert_eq!(
        SerializedValueSlice::new(&[ValueKind::None.into(), 0]).validate(),
        Err(DeserializeError::TrailingData)
    );

    let mut too_deep = vec![ValueKind::Some.into(); MAX_VALUE_DEPTH as usize];
    too_deep.push(ValueKind::None.into());
    assert_eq!(
        SerializedValueSlice::new(&too_deep).validate(),
        Err(DeserializeError::TooDeeplyNested)
    );
}
