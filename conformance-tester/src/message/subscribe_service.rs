use crate::context::Context;
use crate::serial::Serial;
use crate::uuid_ref::UuidRef;
use aldrin_core::message;
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct SubscribeService {
    pub serial: Serial,
    pub service_cookie: UuidRef,
}

impl SubscribeService {
    pub fn to_core(&self, ctx: &Context) -> Result<message::SubscribeService> {
        let serial = self.serial.get(ctx)?;
        let service_cookie = self.service_cookie.get(ctx)?.into();

        Ok(message::SubscribeService {
            serial,
            service_cookie,
        })
    }

    pub fn matches(&self, other: &Self, ctx: &Context) -> Result<bool> {
        let res = self.serial.matches(&other.serial, ctx)?
            && self.service_cookie.matches(&other.service_cookie, ctx)?;

        Ok(res)
    }

    pub fn update_context(&self, other: &Self, ctx: &mut Context) -> Result<()> {
        self.serial.update_context(&other.serial, ctx)?;

        self.service_cookie
            .update_context(&other.service_cookie, ctx)?;

        Ok(())
    }

    pub fn apply_context(&self, ctx: &Context) -> Result<Self> {
        let serial = self.serial.apply_context(ctx)?;
        let service_cookie = self.service_cookie.apply_context(ctx)?;

        Ok(Self {
            serial,
            service_cookie,
        })
    }
}

impl TryFrom<message::SubscribeService> for SubscribeService {
    type Error = Error;

    fn try_from(msg: message::SubscribeService) -> Result<Self> {
        Ok(Self {
            serial: msg.serial.into(),
            service_cookie: msg.service_cookie.into(),
        })
    }
}
