use crate::context::Context;
use crate::serial::Serial;
use crate::uuid_ref::UuidRef;
use aldrin_proto::message;
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct QueryObject {
    pub serial: Serial,
    pub uuid: UuidRef,
    pub with_services: bool,
}

impl QueryObject {
    pub fn to_proto(&self, ctx: &Context) -> Result<message::QueryObject> {
        let serial = self.serial.get(ctx)?;
        let uuid = self.uuid.get(ctx)?.into();

        Ok(message::QueryObject {
            serial,
            uuid,
            with_services: self.with_services,
        })
    }

    pub fn matches(&self, other: &Self, ctx: &Context) -> Result<bool> {
        let res = self.serial.matches(&other.serial, ctx)?
            && self.uuid.matches(&other.uuid, ctx)?
            && (self.with_services == other.with_services);

        Ok(res)
    }

    pub fn update_context(&self, other: &Self, ctx: &mut Context) -> Result<()> {
        self.serial.update_context(&other.serial, ctx)?;
        self.uuid.update_context(&other.uuid, ctx)?;

        Ok(())
    }

    pub fn apply_context(&self, ctx: &Context) -> Result<Self> {
        let serial = self.serial.apply_context(ctx)?;
        let uuid = self.uuid.apply_context(ctx)?;

        Ok(Self {
            serial,
            uuid,
            with_services: self.with_services,
        })
    }
}

impl TryFrom<message::QueryObject> for QueryObject {
    type Error = Error;

    fn try_from(msg: message::QueryObject) -> Result<Self> {
        Ok(Self {
            serial: msg.serial.into(),
            uuid: msg.uuid.into(),
            with_services: msg.with_services,
        })
    }
}
