use super::ServiceInfo;
use crate::context::Context;
use crate::serial::Serial;
use crate::uuid_ref::UuidRef;
use aldrin_core::{message, SerializedValue, ServiceInfo as CoreServiceInfo};
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct CreateService2 {
    pub serial: Serial,
    pub object_cookie: UuidRef,
    pub uuid: UuidRef,
    pub info: Option<ServiceInfo>,
}

impl CreateService2 {
    pub(crate) fn to_core(&self, ctx: &Context) -> Result<message::CreateService2> {
        let serial = self.serial.get(ctx)?;
        let object_cookie = self.object_cookie.get(ctx)?.into();
        let uuid = self.uuid.get(ctx)?.into();

        let value = match self.info {
            Some(ref info) => SerializedValue::serialize(info.to_core(ctx)?)?,
            None => SerializedValue::serialize(())?,
        };

        Ok(message::CreateService2 {
            serial,
            object_cookie,
            uuid,
            value,
        })
    }

    pub(crate) fn matches(&self, other: &Self, ctx: &Context) -> Result<bool> {
        let res = self.serial.matches(&other.serial, ctx)?
            && self.object_cookie.matches(&other.object_cookie, ctx)?
            && self.uuid.matches(&other.uuid, ctx)?;

        let res = match (&self.info, &other.info) {
            (Some(info1), Some(info2)) => res && info1.matches(info2, ctx)?,
            (None, None) => res,
            _ => false,
        };

        Ok(res)
    }

    pub(crate) fn update_context(&self, other: &Self, ctx: &mut Context) -> Result<()> {
        self.serial.update_context(&other.serial, ctx)?;
        self.object_cookie
            .update_context(&other.object_cookie, ctx)?;
        self.uuid.update_context(&other.uuid, ctx)?;

        if let (Some(info1), Some(info2)) = (&self.info, &other.info) {
            info1.update_context(info2, ctx)?;
        }

        Ok(())
    }

    pub(crate) fn apply_context(&self, ctx: &Context) -> Result<Self> {
        let serial = self.serial.apply_context(ctx)?;
        let object_cookie = self.object_cookie.apply_context(ctx)?;
        let uuid = self.uuid.apply_context(ctx)?;

        let info = self
            .info
            .as_ref()
            .map(|info| info.apply_context(ctx))
            .transpose()?;

        Ok(Self {
            serial,
            object_cookie,
            uuid,
            info,
        })
    }
}

impl TryFrom<message::CreateService2> for CreateService2 {
    type Error = Error;

    fn try_from(msg: message::CreateService2) -> Result<Self> {
        Ok(Self {
            serial: msg.serial.into(),
            object_cookie: msg.object_cookie.into(),
            uuid: msg.uuid.into(),

            info: msg
                .value
                .deserialize::<CoreServiceInfo>()
                .map(Into::into)
                .ok(),
        })
    }
}
