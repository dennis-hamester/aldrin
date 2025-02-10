use crate::{
    Deserialize, DeserializeError, Deserializer, PrimaryTag, Serialize, SerializeError, Serializer,
    Value,
};

impl PrimaryTag for String {
    type Tag = Self;
}

impl Serialize<Self> for String {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize::<Self, _>(&self)
    }
}

impl Deserialize<Self> for String {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize_string()
    }
}

impl Serialize<String> for &String {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_string(self)
    }
}

impl Serialize<Value> for String {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize::<Self, _>(self)
    }
}

impl Deserialize<Value> for String {
    fn deserialize(deserializer: Deserializer) -> Result<Self, DeserializeError> {
        deserializer.deserialize::<Self, _>()
    }
}

impl Serialize<Value> for &String {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize::<String, _>(self)
    }
}

impl PrimaryTag for &str {
    type Tag = String;
}

impl Serialize<String> for &str {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize_string(self)
    }
}

impl Serialize<Value> for &str {
    fn serialize(self, serializer: Serializer) -> Result<(), SerializeError> {
        serializer.serialize::<String, _>(self)
    }
}
