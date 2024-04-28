use crate::context::Context;
use crate::serial::Serial;
use crate::uuid_ref::UuidRef;
use aldrin_core::message;
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct QueryIntrospection {
    pub serial: Serial,
    pub type_id: UuidRef,
}

impl QueryIntrospection {
    pub fn to_core(&self, ctx: &Context) -> Result<message::QueryIntrospection> {
        let serial = self.serial.get(ctx)?;
        let type_id = self.type_id.get(ctx)?.into();

        Ok(message::QueryIntrospection { serial, type_id })
    }

    pub fn matches(&self, other: &Self, ctx: &Context) -> Result<bool> {
        let res = self.serial.matches(&other.serial, ctx)?
            && self.type_id.matches(&other.type_id, ctx)?;

        Ok(res)
    }

    pub fn update_context(&self, other: &Self, ctx: &mut Context) -> Result<()> {
        self.serial.update_context(&other.serial, ctx)?;
        self.type_id.update_context(&other.type_id, ctx)?;

        Ok(())
    }

    pub fn apply_context(&self, ctx: &Context) -> Result<Self> {
        let serial = self.serial.apply_context(ctx)?;
        let type_id = self.type_id.apply_context(ctx)?;

        Ok(Self { serial, type_id })
    }
}

impl TryFrom<message::QueryIntrospection> for QueryIntrospection {
    type Error = Error;

    fn try_from(msg: message::QueryIntrospection) -> Result<Self> {
        Ok(Self {
            serial: msg.serial.into(),
            type_id: msg.type_id.into(),
        })
    }
}
