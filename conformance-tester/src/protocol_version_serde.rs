use aldrin_core::ProtocolVersion;
use serde::Deserializer;
use serde::de::{Error, Unexpected, Visitor};
use std::fmt;

pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<ProtocolVersion, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_string(VersionVisitor)
}

pub(crate) fn deserialize_option<'de, D>(
    deserializer: D,
) -> Result<Option<ProtocolVersion>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_option(OptionalVersionVisitor)
}

struct VersionVisitor;

impl Visitor<'_> for VersionVisitor {
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

struct OptionalVersionVisitor;

impl<'de> Visitor<'de> for OptionalVersionVisitor {
    type Value = Option<ProtocolVersion>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "a protocol version of the form MAJOR.MINOR or none"
        )
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_string(VersionVisitor).map(Some)
    }
}
