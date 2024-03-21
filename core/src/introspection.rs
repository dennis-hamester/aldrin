use crate::error::{DeserializeError, SerializeError};
use crate::ids::ServiceUuid;
use crate::serialized_value::SerializedValue;
use crate::value_deserializer::{Deserialize, Deserializer};
use crate::value_serializer::{Serialize, Serializer};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::collections::BTreeMap;
use uuid::{uuid, Uuid};

pub const TYPE_ID_NAMESPACE_SERVICE: Uuid = uuid!("ed4752fa-db9f-4f19-be92-ed7863ca7a9d");
pub const TYPE_ID_NAMESPACE_STRUCT: Uuid = uuid!("83742d78-4e60-44b2-84e7-75904c5987c1");
pub const TYPE_ID_NAMESPACE_INLINE_STRUCT: Uuid = uuid!("56a3e7bb-c2db-4ab9-b411-940d19b0119d");
pub const TYPE_ID_NAMESPACE_ENUM: Uuid = uuid!("4d676655-a5be-491c-bf81-a1b21201589d");
pub const TYPE_ID_NAMESPACE_INLINE_ENUM: Uuid = uuid!("e03eec84-34c9-4e4d-a70b-6dffc124459f");

pub trait Introspectable {
    fn introspection() -> &'static Introspection;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Introspection {
    Service(Service),
    Struct(Struct),
    InlineStruct(InlineStruct),
    Enum(Enum),
    InlineEnum(InlineEnum),
}

impl Introspection {
    pub fn type_id(&self) -> TypeId {
        match self {
            Self::Service(inner) => inner.type_id,
            Self::Struct(inner) => inner.type_id,
            Self::InlineStruct(inner) => inner.type_id,
            Self::Enum(inner) => inner.type_id,
            Self::InlineEnum(inner) => inner.type_id,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
enum IntrospectionVariant {
    Service = 0,
    Struct = 1,
    InlineStruct = 2,
    Enum = 3,
    InlineEnum = 4,
}

impl Serialize for Introspection {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        match self {
            Self::Service(t) => serializer.serialize_enum(IntrospectionVariant::Service.into(), t),
            Self::Struct(t) => serializer.serialize_enum(IntrospectionVariant::Struct.into(), t),
            Self::InlineStruct(t) => {
                serializer.serialize_enum(IntrospectionVariant::InlineStruct.into(), t)
            }
            Self::Enum(t) => serializer.serialize_enum(IntrospectionVariant::Enum.into(), t),
            Self::InlineEnum(t) => {
                serializer.serialize_enum(IntrospectionVariant::InlineEnum.into(), t)
            }
        }
    }
}

impl Deserialize for Introspection {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let deserializer = deserializer.deserialize_enum()?;

        let variant = deserializer
            .variant()
            .try_into()
            .map_err(|_| DeserializeError::InvalidSerialization)?;

        match variant {
            IntrospectionVariant::Service => deserializer.deserialize().map(Self::Service),
            IntrospectionVariant::Struct => deserializer.deserialize().map(Self::Struct),
            IntrospectionVariant::InlineStruct => {
                deserializer.deserialize().map(Self::InlineStruct)
            }
            IntrospectionVariant::Enum => deserializer.deserialize().map(Self::Enum),
            IntrospectionVariant::InlineEnum => deserializer.deserialize().map(Self::InlineEnum),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Service {
    pub type_id: TypeId,
    pub name: String,
    pub uuid: ServiceUuid,
    pub version: u32,
    pub functions: BTreeMap<u32, Function>,
    pub events: BTreeMap<u32, Event>,
}

impl Service {
    pub fn builder(name: impl Into<String>, uuid: ServiceUuid, version: u32) -> ServiceBuilder {
        ServiceBuilder::new(name, uuid, version)
    }

    pub fn set_type_id(&mut self) {
        self.type_id = TypeId::nil();
        self.type_id = compute_type_id(self, TYPE_ID_NAMESPACE_SERVICE);
    }
}

impl From<Service> for Introspection {
    fn from(s: Service) -> Self {
        Self::Service(s)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
enum ServiceField {
    TypeId = 0,
    Name = 1,
    Uuid = 2,
    Version = 3,
    Functions = 4,
    Events = 5,
}

impl Serialize for Service {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct(6)?;

        serializer.serialize_field(ServiceField::TypeId.into(), &self.type_id)?;
        serializer.serialize_field(ServiceField::Name.into(), &self.name)?;
        serializer.serialize_field(ServiceField::Uuid.into(), &self.uuid)?;
        serializer.serialize_field(ServiceField::Version.into(), &self.version)?;
        serializer.serialize_field(ServiceField::Functions.into(), &self.functions)?;
        serializer.serialize_field(ServiceField::Events.into(), &self.events)?;

        Ok(())
    }
}

impl Deserialize for Service {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let mut deserializer = deserializer.deserialize_struct()?;

        let type_id = deserializer.deserialize_field()?;
        if type_id.id() != ServiceField::TypeId.into() {
            return Err(DeserializeError::InvalidSerialization);
        }
        let type_id = type_id.deserialize()?;

        let name = deserializer.deserialize_field()?;
        if name.id() != ServiceField::Name.into() {
            return Err(DeserializeError::InvalidSerialization);
        }
        let name = name.deserialize()?;

        let uuid = deserializer.deserialize_field()?;
        if uuid.id() != ServiceField::Uuid.into() {
            return Err(DeserializeError::InvalidSerialization);
        }
        let uuid = uuid.deserialize()?;

        let version = deserializer.deserialize_field()?;
        if version.id() != ServiceField::Version.into() {
            return Err(DeserializeError::InvalidSerialization);
        }
        let version = version.deserialize()?;

        let functions = deserializer.deserialize_field()?;
        if functions.id() != ServiceField::Functions.into() {
            return Err(DeserializeError::InvalidSerialization);
        }
        let functions = functions.deserialize()?;

        let events = deserializer.deserialize_field()?;
        if events.id() != ServiceField::Events.into() {
            return Err(DeserializeError::InvalidSerialization);
        }
        let events = events.deserialize()?;

        if deserializer.has_more_fields() {
            return Err(DeserializeError::InvalidSerialization);
        }

        Ok(Self {
            type_id,
            name,
            uuid,
            version,
            functions,
            events,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ServiceBuilder {
    inner: Service,
}

impl ServiceBuilder {
    pub fn new(name: impl Into<String>, uuid: ServiceUuid, version: u32) -> Self {
        Self {
            inner: Service {
                type_id: TypeId::nil(),
                name: name.into(),
                uuid,
                version,
                functions: BTreeMap::new(),
                events: BTreeMap::new(),
            },
        }
    }

    pub fn finish(mut self) -> Service {
        self.inner.set_type_id();
        self.inner
    }

    pub fn function(mut self, function: Function) -> Self {
        self.inner.functions.insert(function.id, function);
        self
    }

    pub fn event(mut self, event: Event) -> Self {
        self.inner.events.insert(event.id, event);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Function {
    pub id: u32,
    pub name: String,
    pub args: Option<TypeRef>,
    pub ok: Option<TypeRef>,
    pub err: Option<TypeRef>,
}

impl Function {
    pub fn new(
        id: u32,
        name: impl Into<String>,
        args: Option<TypeRef>,
        ok: Option<TypeRef>,
        err: Option<TypeRef>,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            args,
            ok,
            err,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
enum FunctionField {
    Id = 0,
    Name = 1,
    Args = 2,
    Ok = 3,
    Err = 4,
}

impl Serialize for Function {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct(5)?;

        serializer.serialize_field(FunctionField::Id.into(), &self.id)?;
        serializer.serialize_field(FunctionField::Name.into(), &self.name)?;
        serializer.serialize_field(FunctionField::Args.into(), &self.args)?;
        serializer.serialize_field(FunctionField::Ok.into(), &self.ok)?;
        serializer.serialize_field(FunctionField::Err.into(), &self.err)?;

        Ok(())
    }
}

impl Deserialize for Function {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let mut deserializer = deserializer.deserialize_struct()?;

        let id = deserializer.deserialize_field()?;
        if id.id() != FunctionField::Id.into() {
            return Err(DeserializeError::InvalidSerialization);
        }
        let id = id.deserialize()?;

        let name = deserializer.deserialize_field()?;
        if name.id() != FunctionField::Name.into() {
            return Err(DeserializeError::InvalidSerialization);
        }
        let name = name.deserialize()?;

        let args = deserializer.deserialize_field()?;
        if args.id() != FunctionField::Args.into() {
            return Err(DeserializeError::InvalidSerialization);
        }
        let args = args.deserialize()?;

        let ok = deserializer.deserialize_field()?;
        if ok.id() != FunctionField::Ok.into() {
            return Err(DeserializeError::InvalidSerialization);
        }
        let ok = ok.deserialize()?;

        let err = deserializer.deserialize_field()?;
        if err.id() != FunctionField::Err.into() {
            return Err(DeserializeError::InvalidSerialization);
        }
        let err = err.deserialize()?;

        if deserializer.has_more_fields() {
            return Err(DeserializeError::InvalidSerialization);
        }

        Ok(Self {
            id,
            name,
            args,
            ok,
            err,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    pub id: u32,
    pub name: String,
    pub data: Option<TypeRef>,
}

impl Event {
    pub fn new(id: u32, name: impl Into<String>, data: Option<TypeRef>) -> Self {
        Self {
            id,
            name: name.into(),
            data,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
enum EventField {
    Id = 0,
    Name = 1,
    Data = 2,
}

impl Serialize for Event {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct(3)?;

        serializer.serialize_field(EventField::Id.into(), &self.id)?;
        serializer.serialize_field(EventField::Name.into(), &self.name)?;
        serializer.serialize_field(EventField::Data.into(), &self.data)?;

        Ok(())
    }
}

impl Deserialize for Event {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let mut deserializer = deserializer.deserialize_struct()?;

        let id = deserializer.deserialize_field()?;
        if id.id() != EventField::Id.into() {
            return Err(DeserializeError::InvalidSerialization);
        }
        let id = id.deserialize()?;

        let name = deserializer.deserialize_field()?;
        if name.id() != EventField::Name.into() {
            return Err(DeserializeError::InvalidSerialization);
        }
        let name = name.deserialize()?;

        let data = deserializer.deserialize_field()?;
        if data.id() != EventField::Data.into() {
            return Err(DeserializeError::InvalidSerialization);
        }
        let data = data.deserialize()?;

        if deserializer.has_more_fields() {
            return Err(DeserializeError::InvalidSerialization);
        }

        Ok(Self { id, name, data })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Struct {
    pub type_id: TypeId,
    pub name: String,
    pub fields: BTreeMap<u32, Field>,
}

impl Struct {
    pub fn builder(name: impl Into<String>) -> StructBuilder {
        StructBuilder::new(name)
    }

    pub fn set_type_id(&mut self) {
        self.type_id = TypeId::nil();
        self.type_id = compute_type_id(self, TYPE_ID_NAMESPACE_STRUCT);
    }
}

impl From<Struct> for Introspection {
    fn from(s: Struct) -> Self {
        Self::Struct(s)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
enum StructField {
    TypeId = 0,
    Name = 1,
    Fields = 2,
}

impl Serialize for Struct {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct(3)?;

        serializer.serialize_field(StructField::TypeId.into(), &self.type_id)?;
        serializer.serialize_field(StructField::Name.into(), &self.name)?;
        serializer.serialize_field(StructField::Fields.into(), &self.fields)?;

        Ok(())
    }
}

impl Deserialize for Struct {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let mut deserializer = deserializer.deserialize_struct()?;

        let type_id = deserializer.deserialize_field()?;
        if type_id.id() != StructField::TypeId.into() {
            return Err(DeserializeError::InvalidSerialization);
        }
        let type_id = type_id.deserialize()?;

        let name = deserializer.deserialize_field()?;
        if name.id() != StructField::Name.into() {
            return Err(DeserializeError::InvalidSerialization);
        }
        let name = name.deserialize()?;

        let fields = deserializer.deserialize_field()?;
        if fields.id() != StructField::Fields.into() {
            return Err(DeserializeError::InvalidSerialization);
        }
        let fields = fields.deserialize()?;

        if deserializer.has_more_fields() {
            return Err(DeserializeError::InvalidSerialization);
        }

        Ok(Self {
            type_id,
            name,
            fields,
        })
    }
}

#[derive(Debug, Clone)]
pub struct StructBuilder {
    inner: Struct,
}

impl StructBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            inner: Struct {
                type_id: TypeId::nil(),
                name: name.into(),
                fields: BTreeMap::new(),
            },
        }
    }

    pub fn finish(mut self) -> Struct {
        self.inner.set_type_id();
        self.inner
    }

    pub fn field(mut self, field: Field) -> Self {
        self.inner.fields.insert(field.id, field);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InlineStruct {
    pub type_id: TypeId,
    pub fields: BTreeMap<u32, Field>,
}

impl InlineStruct {
    pub fn builder() -> InlineStructBuilder {
        InlineStructBuilder::new()
    }

    pub fn set_type_id(&mut self) {
        self.type_id = TypeId::nil();
        self.type_id = compute_type_id(self, TYPE_ID_NAMESPACE_INLINE_STRUCT);
    }
}

impl From<InlineStruct> for Introspection {
    fn from(s: InlineStruct) -> Self {
        Self::InlineStruct(s)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
enum InlineStructField {
    TypeId = 0,
    Fields = 1,
}

impl Serialize for InlineStruct {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct(2)?;

        serializer.serialize_field(InlineStructField::TypeId.into(), &self.type_id)?;
        serializer.serialize_field(InlineStructField::Fields.into(), &self.fields)?;

        Ok(())
    }
}

impl Deserialize for InlineStruct {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let mut deserializer = deserializer.deserialize_struct()?;

        let type_id = deserializer.deserialize_field()?;
        if type_id.id() != InlineStructField::TypeId.into() {
            return Err(DeserializeError::InvalidSerialization);
        }
        let type_id = type_id.deserialize()?;

        let fields = deserializer.deserialize_field()?;
        if fields.id() != InlineStructField::Fields.into() {
            return Err(DeserializeError::InvalidSerialization);
        }
        let fields = fields.deserialize()?;

        if deserializer.has_more_fields() {
            return Err(DeserializeError::InvalidSerialization);
        }

        Ok(Self { type_id, fields })
    }
}

#[derive(Debug, Clone)]
pub struct InlineStructBuilder {
    inner: InlineStruct,
}

impl InlineStructBuilder {
    pub fn new() -> Self {
        Self {
            inner: InlineStruct {
                type_id: TypeId::nil(),
                fields: BTreeMap::new(),
            },
        }
    }

    pub fn finish(mut self) -> InlineStruct {
        self.inner.set_type_id();
        self.inner
    }

    pub fn field(mut self, field: Field) -> Self {
        self.inner.fields.insert(field.id, field);
        self
    }
}

impl Default for InlineStructBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    pub id: u32,
    pub name: String,
    pub required: bool,
    pub data: TypeRef,
}

impl Field {
    pub fn new(id: u32, name: impl Into<String>, required: bool, data: TypeRef) -> Self {
        Self {
            id,
            name: name.into(),
            required,
            data,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
enum FieldField {
    Id = 0,
    Name = 1,
    Required = 2,
    Data = 3,
}

impl Serialize for Field {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct(4)?;

        serializer.serialize_field(FieldField::Id.into(), &self.id)?;
        serializer.serialize_field(FieldField::Name.into(), &self.name)?;
        serializer.serialize_field(FieldField::Required.into(), &self.required)?;
        serializer.serialize_field(FieldField::Data.into(), &self.data)?;

        Ok(())
    }
}

impl Deserialize for Field {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let mut deserializer = deserializer.deserialize_struct()?;

        let id = deserializer.deserialize_field()?;
        if id.id() != FieldField::Id.into() {
            return Err(DeserializeError::InvalidSerialization);
        }
        let id = id.deserialize()?;

        let name = deserializer.deserialize_field()?;
        if name.id() != FieldField::Name.into() {
            return Err(DeserializeError::InvalidSerialization);
        }
        let name = name.deserialize()?;

        let required = deserializer.deserialize_field()?;
        if required.id() != FieldField::Required.into() {
            return Err(DeserializeError::InvalidSerialization);
        }
        let required = required.deserialize()?;

        let data = deserializer.deserialize_field()?;
        if data.id() != FieldField::Data.into() {
            return Err(DeserializeError::InvalidSerialization);
        }
        let data = data.deserialize()?;

        if deserializer.has_more_fields() {
            return Err(DeserializeError::InvalidSerialization);
        }

        Ok(Self {
            id,
            name,
            required,
            data,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Enum {
    pub type_id: TypeId,
    pub name: String,
    pub variants: BTreeMap<u32, Variant>,
}

impl Enum {
    pub fn builder(name: impl Into<String>) -> EnumBuilder {
        EnumBuilder::new(name)
    }

    pub fn set_type_id(&mut self) {
        self.type_id = TypeId::nil();
        self.type_id = compute_type_id(self, TYPE_ID_NAMESPACE_ENUM);
    }
}

impl From<Enum> for Introspection {
    fn from(s: Enum) -> Self {
        Self::Enum(s)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
enum EnumField {
    TypeId = 0,
    Name = 1,
    Variants = 2,
}

impl Serialize for Enum {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct(3)?;

        serializer.serialize_field(EnumField::TypeId.into(), &self.type_id)?;
        serializer.serialize_field(EnumField::Name.into(), &self.name)?;
        serializer.serialize_field(EnumField::Variants.into(), &self.variants)?;

        Ok(())
    }
}

impl Deserialize for Enum {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let mut deserializer = deserializer.deserialize_struct()?;

        let type_id = deserializer.deserialize_field()?;
        if type_id.id() != EnumField::TypeId.into() {
            return Err(DeserializeError::InvalidSerialization);
        }
        let type_id = type_id.deserialize()?;

        let name = deserializer.deserialize_field()?;
        if name.id() != EnumField::Name.into() {
            return Err(DeserializeError::InvalidSerialization);
        }
        let name = name.deserialize()?;

        let variants = deserializer.deserialize_field()?;
        if variants.id() != EnumField::Variants.into() {
            return Err(DeserializeError::InvalidSerialization);
        }
        let variants = variants.deserialize()?;

        if deserializer.has_more_fields() {
            return Err(DeserializeError::InvalidSerialization);
        }

        Ok(Self {
            type_id,
            name,
            variants,
        })
    }
}

#[derive(Debug, Clone)]
pub struct EnumBuilder {
    inner: Enum,
}

impl EnumBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            inner: Enum {
                type_id: TypeId::nil(),
                name: name.into(),
                variants: BTreeMap::new(),
            },
        }
    }

    pub fn finish(mut self) -> Enum {
        self.inner.set_type_id();
        self.inner
    }

    pub fn variant(mut self, variant: Variant) -> Self {
        self.inner.variants.insert(variant.id, variant);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InlineEnum {
    pub type_id: TypeId,
    pub variants: BTreeMap<u32, Variant>,
}

impl InlineEnum {
    pub fn builder() -> InlineEnumBuilder {
        InlineEnumBuilder::new()
    }

    pub fn set_type_id(&mut self) {
        self.type_id = TypeId::nil();
        self.type_id = compute_type_id(self, TYPE_ID_NAMESPACE_INLINE_ENUM);
    }
}

impl From<InlineEnum> for Introspection {
    fn from(s: InlineEnum) -> Self {
        Self::InlineEnum(s)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
enum InlineEnumField {
    TypeId = 0,
    Variants = 1,
}

impl Serialize for InlineEnum {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct(2)?;

        serializer.serialize_field(InlineEnumField::TypeId.into(), &self.type_id)?;
        serializer.serialize_field(InlineEnumField::Variants.into(), &self.variants)?;

        Ok(())
    }
}

impl Deserialize for InlineEnum {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let mut deserializer = deserializer.deserialize_struct()?;

        let type_id = deserializer.deserialize_field()?;
        if type_id.id() != InlineEnumField::TypeId.into() {
            return Err(DeserializeError::InvalidSerialization);
        }
        let type_id = type_id.deserialize()?;

        let variants = deserializer.deserialize_field()?;
        if variants.id() != InlineEnumField::Variants.into() {
            return Err(DeserializeError::InvalidSerialization);
        }
        let variants = variants.deserialize()?;

        if deserializer.has_more_fields() {
            return Err(DeserializeError::InvalidSerialization);
        }

        Ok(Self { type_id, variants })
    }
}

#[derive(Debug, Clone)]
pub struct InlineEnumBuilder {
    inner: InlineEnum,
}

impl InlineEnumBuilder {
    pub fn new() -> Self {
        Self {
            inner: InlineEnum {
                type_id: TypeId::nil(),
                variants: BTreeMap::new(),
            },
        }
    }

    pub fn finish(mut self) -> InlineEnum {
        self.inner.set_type_id();
        self.inner
    }

    pub fn variant(mut self, variant: Variant) -> Self {
        self.inner.variants.insert(variant.id, variant);
        self
    }
}

impl Default for InlineEnumBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Variant {
    pub id: u32,
    pub name: String,
    pub data: Option<TypeRef>,
}

impl Variant {
    pub fn new(id: u32, name: impl Into<String>, data: Option<TypeRef>) -> Self {
        Self {
            id,
            name: name.into(),
            data,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
enum VariantField {
    Id = 0,
    Name = 1,
    Data = 2,
}

impl Serialize for Variant {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct(3)?;

        serializer.serialize_field(VariantField::Id.into(), &self.id)?;
        serializer.serialize_field(VariantField::Name.into(), &self.name)?;
        serializer.serialize_field(VariantField::Data.into(), &self.data)?;

        Ok(())
    }
}

impl Deserialize for Variant {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let mut deserializer = deserializer.deserialize_struct()?;

        let id = deserializer.deserialize_field()?;
        if id.id() != VariantField::Id.into() {
            return Err(DeserializeError::InvalidSerialization);
        }
        let id = id.deserialize()?;

        let name = deserializer.deserialize_field()?;
        if name.id() != VariantField::Name.into() {
            return Err(DeserializeError::InvalidSerialization);
        }
        let name = name.deserialize()?;

        let data = deserializer.deserialize_field()?;
        if data.id() != VariantField::Data.into() {
            return Err(DeserializeError::InvalidSerialization);
        }
        let data = data.deserialize()?;

        if deserializer.has_more_fields() {
            return Err(DeserializeError::InvalidSerialization);
        }

        Ok(Self { id, name, data })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypeId(pub Uuid);

impl TypeId {
    pub fn nil() -> Self {
        Self(Uuid::nil())
    }

    pub fn new(namespace: Uuid, serialized: &[u8]) -> Self {
        Self(Uuid::new_v5(&namespace, serialized))
    }
}

impl From<TypeId> for TypeRef {
    fn from(t: TypeId) -> Self {
        Self::Custom(t)
    }
}

impl Serialize for TypeId {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_uuid(self.0);
        Ok(())
    }
}

impl Deserialize for TypeId {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_uuid().map(Self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeRef {
    BuiltIn(BuiltInType),
    Custom(TypeId),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
enum TypeRefVariant {
    BuiltIn = 0,
    Custom = 1,
}

impl Serialize for TypeRef {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        match self {
            Self::BuiltIn(t) => serializer.serialize_enum(TypeRefVariant::BuiltIn.into(), t),
            Self::Custom(t) => serializer.serialize_enum(TypeRefVariant::Custom.into(), t),
        }
    }
}

impl Deserialize for TypeRef {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let deserializer = deserializer.deserialize_enum()?;

        let variant = deserializer
            .variant()
            .try_into()
            .map_err(|_| DeserializeError::InvalidSerialization)?;

        match variant {
            TypeRefVariant::BuiltIn => deserializer.deserialize().map(Self::BuiltIn),
            TypeRefVariant::Custom => deserializer.deserialize().map(Self::Custom),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuiltInType {
    Bool,
    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
    U64,
    I64,
    F32,
    F64,
    String,
    Uuid,
    ObjectId,
    ServiceId,
    Value,
    Option(Box<TypeRef>),
    Box(Box<TypeRef>),
    Vec(Box<TypeRef>),
    Bytes,
    Map(Box<MapType>),
    Set(KeyType),
    Sender(Box<TypeRef>),
    Receiver(Box<TypeRef>),
    Lifetime,
    Unit,
    Result(Box<ResultType>),
}

impl From<BuiltInType> for TypeRef {
    fn from(t: BuiltInType) -> Self {
        Self::BuiltIn(t)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
enum BuiltInTypeVariant {
    Bool = 0,
    U8 = 1,
    I8 = 2,
    U16 = 3,
    I16 = 4,
    U32 = 5,
    I32 = 6,
    U64 = 7,
    I64 = 8,
    F32 = 9,
    F64 = 10,
    String = 11,
    Uuid = 12,
    ObjectId = 13,
    ServiceId = 14,
    Value = 15,
    Option = 16,
    Box = 17,
    Vec = 18,
    Bytes = 19,
    Map = 20,
    Set = 21,
    Sender = 22,
    Receiver = 23,
    Lifetime = 24,
    Unit = 25,
    Result = 26,
}

impl Serialize for BuiltInType {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        match self {
            Self::Bool => serializer.serialize_enum(BuiltInTypeVariant::Bool.into(), &()),
            Self::U8 => serializer.serialize_enum(BuiltInTypeVariant::U8.into(), &()),
            Self::I8 => serializer.serialize_enum(BuiltInTypeVariant::I8.into(), &()),
            Self::U16 => serializer.serialize_enum(BuiltInTypeVariant::U16.into(), &()),
            Self::I16 => serializer.serialize_enum(BuiltInTypeVariant::I16.into(), &()),
            Self::U32 => serializer.serialize_enum(BuiltInTypeVariant::U32.into(), &()),
            Self::I32 => serializer.serialize_enum(BuiltInTypeVariant::I32.into(), &()),
            Self::U64 => serializer.serialize_enum(BuiltInTypeVariant::U64.into(), &()),
            Self::I64 => serializer.serialize_enum(BuiltInTypeVariant::I64.into(), &()),
            Self::F32 => serializer.serialize_enum(BuiltInTypeVariant::F32.into(), &()),
            Self::F64 => serializer.serialize_enum(BuiltInTypeVariant::F64.into(), &()),
            Self::String => serializer.serialize_enum(BuiltInTypeVariant::String.into(), &()),
            Self::Uuid => serializer.serialize_enum(BuiltInTypeVariant::Uuid.into(), &()),
            Self::ObjectId => serializer.serialize_enum(BuiltInTypeVariant::ObjectId.into(), &()),
            Self::ServiceId => serializer.serialize_enum(BuiltInTypeVariant::ServiceId.into(), &()),
            Self::Value => serializer.serialize_enum(BuiltInTypeVariant::Value.into(), &()),
            Self::Option(t) => serializer.serialize_enum(BuiltInTypeVariant::Option.into(), t),
            Self::Box(t) => serializer.serialize_enum(BuiltInTypeVariant::Box.into(), t),
            Self::Vec(t) => serializer.serialize_enum(BuiltInTypeVariant::Vec.into(), t),
            Self::Bytes => serializer.serialize_enum(BuiltInTypeVariant::Bytes.into(), &()),
            Self::Map(t) => serializer.serialize_enum(BuiltInTypeVariant::Map.into(), t),
            Self::Set(t) => serializer.serialize_enum(BuiltInTypeVariant::Set.into(), t),
            Self::Sender(t) => serializer.serialize_enum(BuiltInTypeVariant::Sender.into(), t),
            Self::Receiver(t) => serializer.serialize_enum(BuiltInTypeVariant::Receiver.into(), t),
            Self::Lifetime => serializer.serialize_enum(BuiltInTypeVariant::Lifetime.into(), &()),
            Self::Unit => serializer.serialize_enum(BuiltInTypeVariant::Unit.into(), &()),
            Self::Result(t) => serializer.serialize_enum(BuiltInTypeVariant::Result.into(), t),
        }
    }
}

impl Deserialize for BuiltInType {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let deserializer = deserializer.deserialize_enum()?;

        let variant = deserializer
            .variant()
            .try_into()
            .map_err(|_| DeserializeError::InvalidSerialization)?;

        match variant {
            BuiltInTypeVariant::Bool => deserializer.deserialize().map(|()| Self::Bool),
            BuiltInTypeVariant::U8 => deserializer.deserialize().map(|()| Self::U8),
            BuiltInTypeVariant::I8 => deserializer.deserialize().map(|()| Self::I8),
            BuiltInTypeVariant::U16 => deserializer.deserialize().map(|()| Self::U16),
            BuiltInTypeVariant::I16 => deserializer.deserialize().map(|()| Self::I16),
            BuiltInTypeVariant::U32 => deserializer.deserialize().map(|()| Self::U32),
            BuiltInTypeVariant::I32 => deserializer.deserialize().map(|()| Self::I32),
            BuiltInTypeVariant::U64 => deserializer.deserialize().map(|()| Self::U64),
            BuiltInTypeVariant::I64 => deserializer.deserialize().map(|()| Self::I64),
            BuiltInTypeVariant::F32 => deserializer.deserialize().map(|()| Self::F32),
            BuiltInTypeVariant::F64 => deserializer.deserialize().map(|()| Self::F64),
            BuiltInTypeVariant::String => deserializer.deserialize().map(|()| Self::String),
            BuiltInTypeVariant::Uuid => deserializer.deserialize().map(|()| Self::Uuid),
            BuiltInTypeVariant::ObjectId => deserializer.deserialize().map(|()| Self::ObjectId),
            BuiltInTypeVariant::ServiceId => deserializer.deserialize().map(|()| Self::ServiceId),
            BuiltInTypeVariant::Value => deserializer.deserialize().map(|()| Self::Value),
            BuiltInTypeVariant::Option => deserializer.deserialize().map(Self::Option),
            BuiltInTypeVariant::Box => deserializer.deserialize().map(Self::Box),
            BuiltInTypeVariant::Vec => deserializer.deserialize().map(Self::Vec),
            BuiltInTypeVariant::Bytes => deserializer.deserialize().map(|()| Self::Bytes),
            BuiltInTypeVariant::Map => deserializer.deserialize().map(Self::Map),
            BuiltInTypeVariant::Set => deserializer.deserialize().map(Self::Set),
            BuiltInTypeVariant::Sender => deserializer.deserialize().map(Self::Sender),
            BuiltInTypeVariant::Receiver => deserializer.deserialize().map(Self::Receiver),
            BuiltInTypeVariant::Lifetime => deserializer.deserialize().map(|()| Self::Lifetime),
            BuiltInTypeVariant::Unit => deserializer.deserialize().map(|()| Self::Unit),
            BuiltInTypeVariant::Result => deserializer.deserialize().map(Self::Result),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapType {
    pub key: KeyType,
    pub value: TypeRef,
}

impl MapType {
    pub fn new(key: KeyType, value: TypeRef) -> Self {
        Self { key, value }
    }
}

impl From<MapType> for BuiltInType {
    fn from(t: MapType) -> Self {
        BuiltInType::Map(Box::new(t))
    }
}

impl From<MapType> for TypeRef {
    fn from(t: MapType) -> Self {
        Self::BuiltIn(t.into())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
enum MapTypeField {
    Key = 0,
    Value = 1,
}

impl Serialize for MapType {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct(2)?;

        serializer.serialize_field(MapTypeField::Key.into(), &self.key)?;
        serializer.serialize_field(MapTypeField::Value.into(), &self.value)?;

        Ok(())
    }
}

impl Deserialize for MapType {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let mut deserializer = deserializer.deserialize_struct()?;

        let key = deserializer.deserialize_field()?;
        if key.id() != MapTypeField::Key.into() {
            return Err(DeserializeError::InvalidSerialization);
        }
        let key = key.deserialize()?;

        let value = deserializer.deserialize_field()?;
        if value.id() != MapTypeField::Value.into() {
            return Err(DeserializeError::InvalidSerialization);
        }
        let value = value.deserialize()?;

        if deserializer.has_more_fields() {
            return Err(DeserializeError::InvalidSerialization);
        }

        Ok(Self { key, value })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
pub enum KeyType {
    U8 = 0,
    I8 = 1,
    U16 = 2,
    I16 = 3,
    U32 = 4,
    I32 = 5,
    U64 = 6,
    I64 = 7,
    String = 8,
    Uuid = 9,
}

impl Serialize for KeyType {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_enum((*self).into(), &())
    }
}

impl Deserialize for KeyType {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let deserializer = deserializer.deserialize_enum()?;

        let this = deserializer
            .variant()
            .try_into()
            .map_err(|_| DeserializeError::InvalidSerialization)?;
        deserializer.deserialize::<()>()?;

        Ok(this)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResultType {
    pub ok: TypeRef,
    pub err: TypeRef,
}

impl ResultType {
    pub fn new(ok: TypeRef, err: TypeRef) -> Self {
        Self { ok, err }
    }
}

impl From<ResultType> for BuiltInType {
    fn from(t: ResultType) -> Self {
        BuiltInType::Result(Box::new(t))
    }
}

impl From<ResultType> for TypeRef {
    fn from(t: ResultType) -> Self {
        Self::BuiltIn(t.into())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
enum ResultTypeField {
    Ok = 0,
    Err = 1,
}

impl Serialize for ResultType {
    fn serialize(&self, serializer: Serializer) -> Result<(), SerializeError> {
        let mut serializer = serializer.serialize_struct(2)?;

        serializer.serialize_field(ResultTypeField::Ok.into(), &self.ok)?;
        serializer.serialize_field(ResultTypeField::Err.into(), &self.err)?;

        Ok(())
    }
}

impl Deserialize for ResultType {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        let mut deserializer = deserializer.deserialize_struct()?;

        let ok = deserializer.deserialize_field()?;
        if ok.id() != ResultTypeField::Ok.into() {
            return Err(DeserializeError::InvalidSerialization);
        }
        let ok = ok.deserialize()?;

        let err = deserializer.deserialize_field()?;
        if err.id() != ResultTypeField::Err.into() {
            return Err(DeserializeError::InvalidSerialization);
        }
        let err = err.deserialize()?;

        if deserializer.has_more_fields() {
            return Err(DeserializeError::InvalidSerialization);
        }

        Ok(Self { ok, err })
    }
}

fn compute_type_id<T>(value: &T, namespace: Uuid) -> TypeId
where
    T: Serialize,
{
    // Unwrap is fine here, because this function is used only for the local types here, which
    // always serialize successfully.
    let serialized = SerializedValue::serialize(value).unwrap();
    TypeId::new(namespace, &serialized)
}
