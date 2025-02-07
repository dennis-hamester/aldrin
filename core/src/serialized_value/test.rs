use super::SerializedValue;
use crate::tags::{PrimaryTag, Tag};
use crate::{Deserialize, DeserializeError, Deserializer, Serialize, SerializeError, Serializer};

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
        fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
            let mut serializer = serializer.serialize_struct(2)?;

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
        fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
            let mut serializer = serializer.serialize_struct(2)?;

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
        fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
            let mut deserializer = deserializer.deserialize_struct()?;

            let mut field1 = None;
            let mut field2 = None;

            while !deserializer.is_empty() {
                let deserializer = deserializer.deserialize()?;

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
        field2: "0x1234".to_string(),
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
