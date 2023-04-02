use crate::context::Context;
use crate::uuid_ref::UuidRef;
use aldrin_proto::{message, ObjectId};
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ObjectDestroyedEvent {
    pub uuid: UuidRef,
    pub cookie: UuidRef,
}

impl ObjectDestroyedEvent {
    pub fn to_proto(&self, ctx: &Context) -> Result<message::ObjectDestroyedEvent> {
        let uuid = self.uuid.get(ctx)?.into();
        let cookie = self.cookie.get(ctx)?.into();

        Ok(message::ObjectDestroyedEvent {
            id: ObjectId::new(uuid, cookie),
        })
    }

    pub fn matches(&self, other: &Self, ctx: &Context) -> Result<bool> {
        let res =
            self.uuid.matches(&other.uuid, ctx)? && self.cookie.matches(&other.cookie, ctx)?;

        Ok(res)
    }

    pub fn update_context(&self, other: &Self, ctx: &mut Context) -> Result<()> {
        self.uuid.update_context(&other.uuid, ctx)?;
        self.cookie.update_context(&other.cookie, ctx)?;

        Ok(())
    }

    pub fn apply_context(&self, ctx: &Context) -> Result<Self> {
        let uuid = self.uuid.apply_context(ctx)?;
        let cookie = self.cookie.apply_context(ctx)?;

        Ok(Self { uuid, cookie })
    }
}

impl TryFrom<message::ObjectDestroyedEvent> for ObjectDestroyedEvent {
    type Error = Error;

    fn try_from(msg: message::ObjectDestroyedEvent) -> Result<Self> {
        Ok(Self {
            uuid: msg.id.uuid.into(),
            cookie: msg.id.cookie.into(),
        })
    }
}
