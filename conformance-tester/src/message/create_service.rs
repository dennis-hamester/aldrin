use crate::context::Context;
use crate::serial::Serial;
use crate::uuid_ref::UuidRef;
use aldrin_core::message;
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct CreateService {
    pub serial: Serial,
    pub object_cookie: UuidRef,
    pub uuid: UuidRef,
    pub version: u32,
}

impl CreateService {
    pub fn to_core(&self, ctx: &Context) -> Result<message::CreateService> {
        let serial = self.serial.get(ctx)?;
        let object_cookie = self.object_cookie.get(ctx)?.into();
        let uuid = self.uuid.get(ctx)?.into();

        Ok(message::CreateService {
            serial,
            object_cookie,
            uuid,
            version: self.version,
        })
    }

    pub fn matches(&self, other: &Self, ctx: &Context) -> Result<bool> {
        let res = self.serial.matches(&other.serial, ctx)?
            && self.object_cookie.matches(&other.object_cookie, ctx)?
            && self.uuid.matches(&other.uuid, ctx)?
            && (self.version == other.version);
        Ok(res)
    }

    pub fn update_context(&self, other: &Self, ctx: &mut Context) -> Result<()> {
        self.serial.update_context(&other.serial, ctx)?;
        self.object_cookie
            .update_context(&other.object_cookie, ctx)?;
        self.uuid.update_context(&other.uuid, ctx)?;
        Ok(())
    }

    pub fn apply_context(&self, ctx: &Context) -> Result<Self> {
        let serial = self.serial.apply_context(ctx)?;
        let object_cookie = self.object_cookie.apply_context(ctx)?;
        let uuid = self.uuid.apply_context(ctx)?;

        Ok(Self {
            serial,
            object_cookie,
            uuid,
            version: self.version,
        })
    }
}

impl TryFrom<message::CreateService> for CreateService {
    type Error = Error;

    fn try_from(msg: message::CreateService) -> Result<Self> {
        Ok(Self {
            serial: msg.serial.into(),
            object_cookie: msg.object_cookie.into(),
            uuid: msg.uuid.into(),
            version: msg.version,
        })
    }
}
