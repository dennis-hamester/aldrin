use crate::context::Context;
use crate::serial::Serial;
use aldrin_proto::message;
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct SubscribeServices {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub serial: Option<Serial>,
}

impl SubscribeServices {
    pub fn to_proto(&self, ctx: &Context) -> Result<message::SubscribeServices> {
        let serial = self.serial.as_ref().map(|s| s.get(ctx)).transpose()?;

        Ok(message::SubscribeServices { serial })
    }

    pub fn matches(&self, other: &Self, ctx: &Context) -> Result<bool> {
        match (self.serial.as_ref(), other.serial.as_ref()) {
            (Some(s1), Some(s2)) => s1.matches(s2, ctx),
            (Some(_), None) | (None, Some(_)) => Ok(false),
            (None, None) => Ok(true),
        }
    }

    pub fn update_context(&self, other: &Self, ctx: &mut Context) -> Result<()> {
        match (self.serial.as_ref(), other.serial.as_ref()) {
            (Some(s1), Some(s2)) => s1.update_context(s2, ctx),
            (Some(_), None) | (None, Some(_)) => unreachable!(),
            (None, None) => Ok(()),
        }
    }

    pub fn apply_context(&self, ctx: &Context) -> Result<Self> {
        let serial = self
            .serial
            .as_ref()
            .map(|s| s.apply_context(ctx))
            .transpose()?;

        Ok(Self { serial })
    }
}

impl TryFrom<message::SubscribeServices> for SubscribeServices {
    type Error = Error;

    fn try_from(msg: message::SubscribeServices) -> Result<Self> {
        Ok(Self {
            serial: msg.serial.map(Serial::from),
        })
    }
}
