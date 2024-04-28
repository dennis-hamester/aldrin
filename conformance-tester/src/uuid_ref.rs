use crate::context::Context;
use aldrin_core::{
    BusListenerCookie, ChannelCookie, ObjectCookie, ObjectUuid, ServiceCookie, ServiceUuid, TypeId,
};
use anyhow::{anyhow, Error, Result};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(into = "String", try_from = "String")]
pub enum UuidRef {
    Const(Uuid),
    Get(String),
    Set(String),
}

impl UuidRef {
    pub fn get(&self, ctx: &Context) -> Result<Uuid> {
        match self {
            Self::Const(uuid) => Ok(*uuid),
            Self::Get(id) => ctx.get_uuid(id),
            Self::Set(_) => Err(anyhow!("cannot use a `set:` UUID")),
        }
    }

    pub fn match_and_update_context(&self, other: &Self, ctx: &mut Context) -> Result<bool> {
        let Self::Const(u2) = other else {
            unreachable!();
        };

        match self {
            Self::Const(u1) => Ok(u1 == u2),
            Self::Get(id) => ctx.get_uuid(id).map(|u1| u1 == *u2),
            Self::Set(id) => ctx.set_uuid(id.clone(), *u2).map(|_| true),
        }
    }

    pub fn matches(&self, other: &Self, ctx: &Context) -> Result<bool> {
        let u1 = match self {
            Self::Const(u1) => *u1,
            Self::Get(id) => ctx.get_uuid(id)?,
            Self::Set(_) => return Ok(true),
        };

        let Self::Const(u2) = other else {
            unreachable!();
        };

        Ok(u1 == *u2)
    }

    pub fn update_context(&self, other: &Self, ctx: &mut Context) -> Result<()> {
        let Self::Set(id) = self else {
            return Ok(());
        };

        let Self::Const(u) = other else {
            unreachable!();
        };

        ctx.set_uuid(id.clone(), *u)
    }

    pub fn apply_context(&self, ctx: &Context) -> Result<Self> {
        match self {
            Self::Const(uuid) => Ok(Self::Const(*uuid)),
            Self::Get(id) => ctx.get_uuid(id).map(Self::Const),
            Self::Set(id) => Ok(Self::Set(id.clone())),
        }
    }
}

impl fmt::Display for UuidRef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Const(uuid) => uuid.fmt(f),
            Self::Get(id) => f.write_fmt(format_args!("get:{id}")),
            Self::Set(id) => f.write_fmt(format_args!("set:{id}")),
        }
    }
}

impl FromStr for UuidRef {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        if let Some((_, id)) = s.split_once("get:") {
            if id.is_empty() {
                Err(anyhow!("the id cannot be empty")
                    .context(anyhow!("failed to parse UUID `{s}`")))
            } else {
                Ok(Self::Get(id.to_owned()))
            }
        } else if let Some((_, id)) = s.split_once("set:") {
            if id.is_empty() {
                Err(anyhow!("the id cannot be empty")
                    .context(anyhow!("failed to parse UUID `{s}`")))
            } else {
                Ok(Self::Set(id.to_owned()))
            }
        } else if let Ok(uuid) = s.parse() {
            Ok(Self::Const(uuid))
        } else {
            Err(anyhow!("failed to parse UUID `{s}`"))
        }
    }
}

impl From<UuidRef> for String {
    fn from(value: UuidRef) -> Self {
        value.to_string()
    }
}

impl TryFrom<String> for UuidRef {
    type Error = Error;

    fn try_from(value: String) -> Result<Self> {
        value.parse()
    }
}

impl From<Uuid> for UuidRef {
    fn from(value: Uuid) -> Self {
        Self::Const(value)
    }
}

impl From<ObjectUuid> for UuidRef {
    fn from(value: ObjectUuid) -> Self {
        value.0.into()
    }
}

impl From<ObjectCookie> for UuidRef {
    fn from(value: ObjectCookie) -> Self {
        value.0.into()
    }
}

impl From<ServiceUuid> for UuidRef {
    fn from(value: ServiceUuid) -> Self {
        value.0.into()
    }
}

impl From<ServiceCookie> for UuidRef {
    fn from(value: ServiceCookie) -> Self {
        value.0.into()
    }
}

impl From<ChannelCookie> for UuidRef {
    fn from(value: ChannelCookie) -> Self {
        value.0.into()
    }
}

impl From<BusListenerCookie> for UuidRef {
    fn from(value: BusListenerCookie) -> Self {
        value.0.into()
    }
}

impl From<TypeId> for UuidRef {
    fn from(value: TypeId) -> Self {
        value.0.into()
    }
}
