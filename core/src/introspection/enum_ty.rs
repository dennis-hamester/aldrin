use super::{Layout, Variant, VariantBuilder};
use crate::error::{DeserializeError, SerializeError};
use crate::value_deserializer::{Deserialize, Deserializer};
use crate::value_serializer::{Serialize, Serializer};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::collections::BTreeMap;
use uuid::{uuid, Uuid};

#[derive(Debug, Clone)]
pub struct Enum {
    name: String,
    variants: BTreeMap<u32, Variant>,
}

impl Enum {
    pub const NAMESPACE: Uuid = uuid!("642bf73e-991f-406a-b55a-ce914d77480b");

    pub fn builder(name: impl Into<String>) -> EnumBuilder {
        EnumBuilder::new(name)
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn variants(&self) -> &BTreeMap<u32, Variant> {
        &self.variants
    }
}

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
enum EnumField {
    Name = 0,
    Variants = 1,
}

impl Serialize for Enum {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct(2)?;

        serializer.serialize_field(EnumField::Name, &self.name)?;
        serializer.serialize_field(EnumField::Variants, &self.variants)?;

        serializer.finish()
    }
}

impl Deserialize for Enum {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let mut deserializer = deserializer.deserialize_struct()?;

        let name = deserializer.deserialize_specific_field(EnumField::Name)?;
        let variants = deserializer.deserialize_specific_field(EnumField::Variants)?;

        deserializer.finish(Self { name, variants })
    }
}

impl From<Enum> for Layout {
    fn from(e: Enum) -> Self {
        Self::Enum(e)
    }
}

#[derive(Debug, Clone)]
pub struct EnumBuilder {
    name: String,
    variants: BTreeMap<u32, Variant>,
}

impl EnumBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            variants: BTreeMap::new(),
        }
    }

    pub fn variant(
        mut self,
        id: u32,
        name: impl Into<String>,
        f: impl FnOnce(VariantBuilder) -> Variant,
    ) -> Self {
        self.variants.insert(id, f(VariantBuilder::new(id, name)));
        self
    }

    pub fn finish(self) -> Enum {
        Enum {
            name: self.name,
            variants: self.variants,
        }
    }
}
