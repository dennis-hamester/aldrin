use super::{LexicalId, ir, resolve_ir};
use crate::tags::{self, PrimaryTag, Tag};
use crate::{
    Deserialize, DeserializeError, Deserializer, Serialize, SerializeError, Serializer, TypeId,
};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case")
)]
pub struct Variant {
    id: u32,
    name: String,

    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    doc: Option<String>,

    #[cfg_attr(
        feature = "serde",
        serde(rename = "type", default, skip_serializing_if = "Option::is_none")
    )]
    variant_type: Option<TypeId>,
}

impl Variant {
    pub fn from_ir(ty: ir::VariantIr, references: &BTreeMap<LexicalId, TypeId>) -> Self {
        Self {
            id: ty.id,
            name: ty.name,
            doc: ty.doc,
            variant_type: ty.variant_type.map(|ty| resolve_ir(ty, references)),
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn doc(&self) -> Option<&str> {
        self.doc.as_deref()
    }

    pub fn variant_type(&self) -> Option<TypeId> {
        self.variant_type
    }
}

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
enum VariantField {
    Id = 0,
    Name = 1,
    Doc = 2,
    VariantType = 3,
}

impl Tag for Variant {}

impl PrimaryTag for Variant {
    type Tag = Self;
}

impl Serialize<Self> for Variant {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize(&self)
    }
}

impl Serialize<Variant> for &Variant {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct2()?;

        serializer.serialize::<tags::U32>(VariantField::Id, &self.id)?;
        serializer.serialize::<tags::String>(VariantField::Name, &self.name)?;
        serializer.serialize_if_some::<tags::Option<tags::String>>(VariantField::Doc, &self.doc)?;

        serializer.serialize_if_some::<tags::Option<TypeId>>(
            VariantField::VariantType,
            &self.variant_type,
        )?;

        serializer.finish()
    }
}

impl Deserialize<Self> for Variant {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let mut deserializer = deserializer.deserialize_struct()?;

        let mut id = None;
        let mut name = None;
        let mut doc = None;
        let mut variant_type = None;

        while let Some(deserializer) = deserializer.deserialize()? {
            match deserializer.try_id() {
                Ok(VariantField::Id) => {
                    id = deserializer.deserialize::<tags::U32, _>().map(Some)?;
                }

                Ok(VariantField::Name) => {
                    name = deserializer.deserialize::<tags::String, _>().map(Some)?;
                }

                Ok(VariantField::Doc) => {
                    doc = deserializer.deserialize::<tags::Option<tags::String>, _>()?;
                }

                Ok(VariantField::VariantType) => {
                    variant_type = deserializer.deserialize::<tags::Option<TypeId>, _>()?;
                }

                Err(_) => deserializer.skip()?,
            }
        }

        deserializer.finish(Self {
            id: id.ok_or(DeserializeError::InvalidSerialization)?,
            name: name.ok_or(DeserializeError::InvalidSerialization)?,
            doc,
            variant_type,
        })
    }
}
