use crate::context::Context;
use crate::serial::Serial;
use crate::uuid_ref::UuidRef;
use aldrin_core::message;
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct SubscribeEvent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub serial: Option<Serial>,

    pub service_cookie: UuidRef,
    pub event: u32,
}

impl SubscribeEvent {
    pub(crate) fn to_core(&self, ctx: &Context) -> Result<message::SubscribeEvent> {
        let serial = self.serial.as_ref().map(|s| s.get(ctx)).transpose()?;
        let service_cookie = self.service_cookie.get(ctx)?.into();

        Ok(message::SubscribeEvent {
            serial,
            service_cookie,
            event: self.event,
        })
    }

    pub(crate) fn matches(&self, other: &Self, ctx: &Context) -> Result<bool> {
        let res = match (self.serial.as_ref(), other.serial.as_ref()) {
            (Some(s1), Some(s2)) => s1.matches(s2, ctx)?,
            (Some(_), None) | (None, Some(_)) => return Ok(false),
            (None, None) => true,
        };

        let res = res
            && self.service_cookie.matches(&other.service_cookie, ctx)?
            && (self.event == other.event);

        Ok(res)
    }

    pub(crate) fn update_context(&self, other: &Self, ctx: &mut Context) -> Result<()> {
        match (self.serial.as_ref(), other.serial.as_ref()) {
            (Some(s1), Some(s2)) => s1.update_context(s2, ctx)?,
            (Some(_), None) | (None, Some(_)) => unreachable!(),
            (None, None) => {}
        }

        self.service_cookie
            .update_context(&other.service_cookie, ctx)?;

        Ok(())
    }

    pub(crate) fn apply_context(&self, ctx: &Context) -> Result<Self> {
        let serial = self
            .serial
            .as_ref()
            .map(|s| s.apply_context(ctx))
            .transpose()?;

        let service_cookie = self.service_cookie.apply_context(ctx)?;

        Ok(Self {
            serial,
            service_cookie,
            event: self.event,
        })
    }
}

impl TryFrom<message::SubscribeEvent> for SubscribeEvent {
    type Error = Error;

    fn try_from(msg: message::SubscribeEvent) -> Result<Self> {
        Ok(Self {
            serial: msg.serial.map(Serial::from),
            service_cookie: msg.service_cookie.into(),
            event: msg.event,
        })
    }
}
