use super::LexicalId;
use crate::tags::{PrimaryTag, Tag};
use crate::{Deserialize, DeserializeError, Deserializer, Serialize, SerializeError, Serializer};
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ResultType {
    ok: LexicalId,
    err: LexicalId,
}

impl ResultType {
    pub fn new(ok: LexicalId, err: LexicalId) -> Self {
        Self { ok, err }
    }

    pub fn ok(self) -> LexicalId {
        self.ok
    }

    pub fn err(self) -> LexicalId {
        self.err
    }
}

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
enum ResultTypeField {
    Ok = 0,
    Err = 1,
}

impl Tag for ResultType {}

impl PrimaryTag for ResultType {
    type Tag = Self;
}

impl Serialize<Self> for ResultType {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct(2)?;

        serializer.serialize::<LexicalId, _>(ResultTypeField::Ok, self.ok)?;
        serializer.serialize::<LexicalId, _>(ResultTypeField::Err, self.err)?;

        serializer.finish()
    }
}

impl Serialize<ResultType> for &ResultType {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize(*self)
    }
}

impl Deserialize<Self> for ResultType {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let mut deserializer = deserializer.deserialize_struct()?;

        let mut ok = None;
        let mut err = None;

        while !deserializer.is_empty() {
            let deserializer = deserializer.deserialize()?;

            match deserializer.try_id() {
                Ok(ResultTypeField::Ok) => {
                    ok = deserializer.deserialize::<LexicalId, _>().map(Some)?
                }

                Ok(ResultTypeField::Err) => {
                    err = deserializer.deserialize::<LexicalId, _>().map(Some)?
                }

                Err(_) => deserializer.skip()?,
            }
        }

        deserializer.finish(Self {
            ok: ok.ok_or(DeserializeError::InvalidSerialization)?,
            err: err.ok_or(DeserializeError::InvalidSerialization)?,
        })
    }
}
