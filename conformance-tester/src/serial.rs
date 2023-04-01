use anyhow::Result;
use serde::de::{Error, Unexpected, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

#[derive(Debug, Clone)]
pub enum Serial {
    Const(u32),
    Get(String),
    Set(String),
}

impl Serialize for Serial {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Const(serial) => serializer.serialize_u32(*serial),
            Self::Get(id) => serializer.serialize_str(&format!("get:{id}")),
            Self::Set(id) => serializer.serialize_str(&format!("set:{id}")),
        }
    }
}

impl<'de> Deserialize<'de> for Serial {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct SerialVisitor;

        impl<'de> Visitor<'de> for SerialVisitor {
            type Value = Serial;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(
                    formatter,
                    "a non-negative integer or a string of the form `get:{{id}}` or `set:{{id}}`"
                )
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: Error,
            {
                match v.try_into() {
                    Ok(v) => Ok(Serial::Const(v)),
                    Err(_) => Err(E::invalid_value(
                        Unexpected::Unsigned(v),
                        &"a value in the range 0-4294967295",
                    )),
                }
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                if let Some((_, id)) = v.split_once("get:") {
                    if id.is_empty() {
                        Err(E::invalid_value(
                            Unexpected::Str(v),
                            &"a non-empty id after `get:`",
                        ))
                    } else {
                        Ok(Serial::Get(id.to_owned()))
                    }
                } else if let Some((_, id)) = v.split_once("set:") {
                    if id.is_empty() {
                        Err(E::invalid_value(
                            Unexpected::Str(v),
                            &"a non-empty id after `set:`",
                        ))
                    } else {
                        Ok(Serial::Set(id.to_owned()))
                    }
                } else {
                    Err(E::invalid_value(
                        Unexpected::Str(v),
                        &"`get:{{id}}` or `set:{{id}}`",
                    ))
                }
            }
        }

        deserializer.deserialize_any(SerialVisitor)
    }
}

impl From<u32> for Serial {
    fn from(value: u32) -> Self {
        Self::Const(value)
    }
}
