use super::ir;
use crate::tags::{self, KeyTag, PrimaryKeyTag, PrimaryTag, Tag};
use crate::{
    Deserialize, DeserializeError, DeserializeKey, Deserializer, Serialize, SerializeError,
    SerializeKey, Serializer, TypeId,
};
use std::fmt;
use std::str::FromStr;
use uuid::{Error as UuidError, Uuid, uuid};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
#[repr(transparent)]
pub struct LexicalId(pub Uuid);

impl LexicalId {
    pub const NIL: Self = Self(Uuid::nil());

    pub const BOOL: Self = Self(uuid!("f00cbfc4-cf2f-457a-aa56-35923c8b2571"));
    pub const U8: Self = Self(uuid!("0ff2b764-1666-46e1-82ff-aa59264ba7f0"));
    pub const I8: Self = Self(uuid!("055a9029-e3ff-4b38-922f-6a5eee338138"));
    pub const U16: Self = Self(uuid!("355797a3-ef55-49d0-a1e3-3048247c9365"));
    pub const I16: Self = Self(uuid!("3e86f004-6b09-46dd-aacd-f1161ad546f4"));
    pub const U32: Self = Self(uuid!("dff83171-4355-4fde-a2a4-67bd804b31c2"));
    pub const I32: Self = Self(uuid!("8afa8119-736a-4bab-ad71-3b6f8061bed0"));
    pub const U64: Self = Self(uuid!("1a192e74-8220-4bad-bacb-3385e9c26abf"));
    pub const I64: Self = Self(uuid!("a4669bfb-1c1c-43c4-ad3f-ea2afab22756"));
    pub const F32: Self = Self(uuid!("046a2593-0627-44bf-8a6c-d24cb7ef54b2"));
    pub const F64: Self = Self(uuid!("64d58c83-68f9-43d2-9401-04dbc61e34b0"));
    pub const STRING: Self = Self(uuid!("034cb183-38c7-4d26-984e-c56730eafc3f"));
    pub const UUID: Self = Self(uuid!("8cdde1d6-e2ca-4e19-860d-cbe901547816"));
    pub const OBJECT_ID: Self = Self(uuid!("abab21a3-2ebe-47d6-9caa-1e4fd71e2171"));
    pub const SERVICE_ID: Self = Self(uuid!("b241ed0d-89db-493e-a4f5-73f13aa4a7a0"));
    pub const VALUE: Self = Self(uuid!("eb70f272-1c31-4933-a933-4030dd012f07"));
    pub const BYTES: Self = Self(uuid!("300d98f6-3267-48c2-8aa0-dc36e28b0c43"));
    pub const LIFETIME: Self = Self(uuid!("e406e363-6eef-41c8-83ae-d114fd6cb0b8"));
    pub const UNIT: Self = Self(uuid!("630e81c6-b8f3-4c0f-97e3-44d214168d6c"));

    pub const NAMESPACE_OPTION: Uuid = uuid!("050c596f-9fcc-4d5b-9caa-3507553bc64a");
    pub const NAMESPACE_BOX: Uuid = uuid!("b3073eb9-7200-44ff-b79e-5165103a9382");
    pub const NAMESPACE_VEC: Uuid = uuid!("c638aee9-5728-42f8-8405-49ae00280a85");
    pub const NAMESPACE_MAP: Uuid = uuid!("69370c52-3211-46d9-8f7d-9ab0fd1536c9");
    pub const NAMESPACE_SET: Uuid = uuid!("d9fa4f35-368c-4ad0-86f1-35ace576d58b");
    pub const NAMESPACE_SENDER: Uuid = uuid!("052ac1dd-c0d4-4f4c-9b7a-78448875a21f");
    pub const NAMESPACE_RECEIVER: Uuid = uuid!("d697238d-56e0-4132-980e-baf1a64c9bfd");
    pub const NAMESPACE_RESULT: Uuid = uuid!("aef81d6c-35cc-43f7-99f3-a17c0eada1f4");
    pub const NAMESPACE_ARRAY: Uuid = uuid!("770f9cf7-be15-454e-9fea-bb452fa813ed");
    pub const NAMESPACE_CUSTOM: Uuid = uuid!("04334fe0-0ea2-44ea-97b2-c17a7a4cbbd3");
    pub const NAMESPACE_SERVICE: Uuid = uuid!("ddd86559-be89-4b6c-a460-fc347cd6f00b");

    pub fn option(ty: Self) -> Self {
        Self::new_v5(Self::NAMESPACE_OPTION, ty.0)
    }

    pub fn box_ty(ty: Self) -> Self {
        Self::new_v5(Self::NAMESPACE_BOX, ty.0)
    }

    pub fn vec(ty: Self) -> Self {
        Self::new_v5(Self::NAMESPACE_VEC, ty.0)
    }

    pub fn map(key: Self, ty: Self) -> Self {
        Self::new_v5_2(Self::NAMESPACE_MAP, key.0, ty.0)
    }

    pub fn set(ty: Self) -> Self {
        Self::new_v5(Self::NAMESPACE_SET, ty.0)
    }

    pub fn sender(ty: Self) -> Self {
        Self::new_v5(Self::NAMESPACE_SENDER, ty.0)
    }

    pub fn receiver(ty: Self) -> Self {
        Self::new_v5(Self::NAMESPACE_RECEIVER, ty.0)
    }

    pub fn result(ok: Self, err: Self) -> Self {
        Self::new_v5_2(Self::NAMESPACE_RESULT, ok.0, err.0)
    }

    pub fn array(ty: Self, len: u32) -> Self {
        let mut name = [0; 20];
        name[..16].copy_from_slice(ty.0.as_bytes());
        name[16..].copy_from_slice(&len.to_le_bytes());
        Self(Uuid::new_v5(&Self::NAMESPACE_ARRAY, &name))
    }

    pub fn custom(schema: impl AsRef<str>, name: impl AsRef<str>) -> Self {
        Self::fully_qualified(Self::NAMESPACE_CUSTOM, schema, name, &[])
    }

    pub fn custom_generic<const N: usize>(
        schema: impl AsRef<str>,
        name: impl AsRef<str>,
        types: &[Self; N],
    ) -> Self {
        Self::fully_qualified(Self::NAMESPACE_CUSTOM, schema, name, types)
    }

    pub fn service(schema: impl AsRef<str>, name: impl AsRef<str>) -> Self {
        Self::fully_qualified(Self::NAMESPACE_SERVICE, schema, name, &[])
    }

    pub const fn is_nil(self) -> bool {
        self.0.is_nil()
    }

    pub fn resolve(self, introspection: &ir::IntrospectionIr) -> Option<TypeId> {
        introspection.resolve(self)
    }

    fn new_v5(ns: Uuid, ty: Uuid) -> Self {
        Self(Uuid::new_v5(&ns, ty.as_bytes()))
    }

    fn new_v5_2(ns: Uuid, a: Uuid, b: Uuid) -> Self {
        let mut name = [0; 32];
        name[..16].copy_from_slice(a.as_bytes());
        name[16..].copy_from_slice(b.as_bytes());
        Self(Uuid::new_v5(&ns, &name))
    }

    fn fully_qualified<const N: usize>(
        ns: Uuid,
        schema: impl AsRef<str>,
        name: impl AsRef<str>,
        types: &[Self; N],
    ) -> Self {
        let mut fully_qualified = format!("{}::{}", schema.as_ref(), name.as_ref());

        if N > 0 {
            fully_qualified.push('<');
        }

        for (i, ty) in types.iter().enumerate() {
            if i > 0 {
                fully_qualified.push(',');
            }

            fully_qualified.push_str(&ty.to_string());
        }

        if N > 0 {
            fully_qualified.push('>');
        }

        Self(Uuid::new_v5(&ns, fully_qualified.as_bytes()))
    }
}

impl Tag for LexicalId {}

impl PrimaryTag for LexicalId {
    type Tag = Self;
}

impl Serialize<Self> for LexicalId {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_uuid(self.0)
    }
}

impl Serialize<LexicalId> for &LexicalId {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize::<LexicalId>(*self)
    }
}

impl Deserialize<Self> for LexicalId {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_uuid().map(Self)
    }
}

impl Serialize<tags::Uuid> for LexicalId {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize::<Self>(self)
    }
}

impl Serialize<tags::Uuid> for &LexicalId {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize::<tags::Uuid>(*self)
    }
}

impl Deserialize<tags::Uuid> for LexicalId {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize::<Self, _>()
    }
}

impl KeyTag for LexicalId {
    type Impl = tags::Uuid;
}

impl PrimaryKeyTag for LexicalId {
    type KeyTag = Self;
}

impl SerializeKey<Self> for LexicalId {
    fn try_as_key(&self) -> Result<Uuid, SerializeError> {
        Ok(self.0)
    }
}

impl DeserializeKey<Self> for LexicalId {
    fn try_from_key(key: Uuid) -> Result<Self, DeserializeError> {
        Ok(Self(key))
    }
}

impl SerializeKey<tags::Uuid> for LexicalId {
    fn try_as_key(&self) -> Result<Uuid, SerializeError> {
        Ok(self.0)
    }
}

impl DeserializeKey<tags::Uuid> for LexicalId {
    fn try_from_key(key: Uuid) -> Result<Self, DeserializeError> {
        Ok(Self(key))
    }
}

impl From<Uuid> for LexicalId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<LexicalId> for Uuid {
    fn from(id: LexicalId) -> Self {
        id.0
    }
}

impl fmt::Display for LexicalId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for LexicalId {
    type Err = UuidError;

    fn from_str(s: &str) -> Result<Self, UuidError> {
        s.parse().map(Self)
    }
}
