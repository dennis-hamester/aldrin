use crate::error::{DeserializeError, SerializeError};
use crate::serialized_value::SerializedValue;
use crate::value_deserializer::{Deserialize, Deserializer};
use crate::value_serializer::{Serialize, Serializer};

#[test]
fn concrete_vs_vague() {
    #[derive(Debug, PartialEq, Eq)]
    struct Type<T1, T2> {
        field1: T1,
        field2: T2,
    }

    impl<T1, T2> Serialize for Type<T1, T2>
    where
        T1: Serialize,
        T2: Serialize,
    {
        fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
            let mut serializer = serializer.serialize_struct(2)?;
            serializer.serialize_field(1u32, &self.field1)?;
            serializer.serialize_field(2u32, &self.field2)?;
            serializer.finish()
        }
    }

    impl<T1, T2> Deserialize for Type<T1, T2>
    where
        T1: Deserialize,
        T2: Deserialize,
    {
        fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
            let mut deserializer = deserializer.deserialize_struct()?;

            let mut field1 = None;
            let mut field2 = None;

            while deserializer.has_more_fields() {
                let deserializer = deserializer.deserialize_field()?;

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
        field1: SerializedValue::serialize(&0x1234).unwrap(),
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
