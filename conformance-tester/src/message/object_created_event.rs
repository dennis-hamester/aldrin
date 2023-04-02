use crate::context::Context;
use crate::serial::Serial;
use crate::uuid_ref::UuidRef;
use aldrin_proto::{message, ObjectId};
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ObjectCreatedEvent {
    pub uuid: UuidRef,
    pub cookie: UuidRef,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub serial: Option<Serial>,
}

impl ObjectCreatedEvent {
    pub fn to_proto(&self, ctx: &Context) -> Result<message::ObjectCreatedEvent> {
        let uuid = self.uuid.get(ctx)?.into();
        let cookie = self.cookie.get(ctx)?.into();
        let serial = self.serial.as_ref().map(|s| s.get(ctx)).transpose()?;

        Ok(message::ObjectCreatedEvent {
            id: ObjectId::new(uuid, cookie),
            serial,
        })
    }

    pub fn matches(&self, other: &Self, ctx: &Context) -> Result<bool> {
        let res = match (self.serial.as_ref(), other.serial.as_ref()) {
            (Some(s1), Some(s2)) => s1.matches(s2, ctx)?,
            (Some(_), None) | (None, Some(_)) => return Ok(false),
            (None, None) => true,
        };

        let res = res
            && self.uuid.matches(&other.uuid, ctx)?
            && self.cookie.matches(&other.cookie, ctx)?;

        Ok(res)
    }

    pub fn update_context(&self, other: &Self, ctx: &mut Context) -> Result<()> {
        self.uuid.update_context(&other.uuid, ctx)?;
        self.cookie.update_context(&other.cookie, ctx)?;

        match (self.serial.as_ref(), other.serial.as_ref()) {
            (Some(s1), Some(s2)) => s1.update_context(s2, ctx)?,
            (Some(_), None) | (None, Some(_)) => unreachable!(),
            (None, None) => {}
        }

        Ok(())
    }

    pub fn apply_context(&self, ctx: &Context) -> Result<Self> {
        let uuid = self.uuid.apply_context(ctx)?;
        let cookie = self.cookie.apply_context(ctx)?;
        let serial = self
            .serial
            .as_ref()
            .map(|s| s.apply_context(ctx))
            .transpose()?;

        Ok(Self {
            uuid,
            cookie,
            serial,
        })
    }
}

impl TryFrom<message::ObjectCreatedEvent> for ObjectCreatedEvent {
    type Error = Error;

    fn try_from(msg: message::ObjectCreatedEvent) -> Result<Self> {
        Ok(Self {
            uuid: msg.id.uuid.into(),
            cookie: msg.id.cookie.into(),
            serial: msg.serial.map(Serial::from),
        })
    }
}
