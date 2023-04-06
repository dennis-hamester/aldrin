use crate::context::Context;
use crate::uuid_ref::UuidRef;
use aldrin_proto::{message, ObjectId, ServiceId};
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ServiceDestroyedEvent {
    pub object_uuid: UuidRef,
    pub object_cookie: UuidRef,
    pub service_uuid: UuidRef,
    pub service_cookie: UuidRef,
}

impl ServiceDestroyedEvent {
    pub fn to_proto(&self, ctx: &Context) -> Result<message::ServiceDestroyedEvent> {
        let object_uuid = self.object_uuid.get(ctx)?.into();
        let object_cookie = self.object_cookie.get(ctx)?.into();
        let service_uuid = self.service_uuid.get(ctx)?.into();
        let service_cookie = self.service_cookie.get(ctx)?.into();

        Ok(message::ServiceDestroyedEvent {
            id: ServiceId::new(
                ObjectId::new(object_uuid, object_cookie),
                service_uuid,
                service_cookie,
            ),
        })
    }

    pub fn matches(&self, other: &Self, ctx: &Context) -> Result<bool> {
        let res = self.object_uuid.matches(&other.object_uuid, ctx)?
            && self.object_cookie.matches(&other.object_cookie, ctx)?
            && self.service_uuid.matches(&other.service_uuid, ctx)?
            && self.service_cookie.matches(&other.service_cookie, ctx)?;

        Ok(res)
    }

    pub fn update_context(&self, other: &Self, ctx: &mut Context) -> Result<()> {
        self.object_uuid.update_context(&other.object_uuid, ctx)?;
        self.object_cookie
            .update_context(&other.object_cookie, ctx)?;
        self.service_uuid.update_context(&other.service_uuid, ctx)?;
        self.service_cookie
            .update_context(&other.service_cookie, ctx)?;

        Ok(())
    }

    pub fn apply_context(&self, ctx: &Context) -> Result<Self> {
        let object_uuid = self.object_uuid.apply_context(ctx)?;
        let object_cookie = self.object_cookie.apply_context(ctx)?;
        let service_uuid = self.service_uuid.apply_context(ctx)?;
        let service_cookie = self.service_cookie.apply_context(ctx)?;

        Ok(Self {
            object_uuid,
            object_cookie,
            service_uuid,
            service_cookie,
        })
    }
}

impl TryFrom<message::ServiceDestroyedEvent> for ServiceDestroyedEvent {
    type Error = Error;

    fn try_from(msg: message::ServiceDestroyedEvent) -> Result<Self> {
        Ok(Self {
            object_uuid: msg.id.object_id.uuid.into(),
            object_cookie: msg.id.object_id.cookie.into(),
            service_uuid: msg.id.uuid.into(),
            service_cookie: msg.id.cookie.into(),
        })
    }
}
