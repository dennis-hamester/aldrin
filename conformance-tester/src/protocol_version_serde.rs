use aldrin_core::ProtocolVersion;
use serde::de::{Error, Unexpected, Visitor};
use serde::Deserializer;
use std::fmt;

pub fn deserialize<'de, D>(deserializer: D) -> Result<ProtocolVersion, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_string(VersionVisitor)
}

struct VersionVisitor;

impl<'de> Visitor<'de> for VersionVisitor {
    type Value = ProtocolVersion;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a protocol version of the form MAJOR.MINOR")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        v.parse()
            .map_err(|_| Error::invalid_value(Unexpected::Str(v), &Self))
    }
}
