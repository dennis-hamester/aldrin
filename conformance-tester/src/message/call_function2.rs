use crate::context::Context;
use crate::serial::Serial;
use crate::uuid_ref::UuidRef;
use crate::value::Value;
use aldrin_core::{SerializedValue, message};
use anyhow::{Context as _, Error, Result, anyhow};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct CallFunction2 {
    pub serial: Serial,
    pub service_cookie: UuidRef,
    pub function: u32,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<u32>,

    #[serde(flatten)]
    pub value: Value,
}

impl CallFunction2 {
    pub(crate) fn to_core(&self, ctx: &Context) -> Result<message::CallFunction2> {
        let serial = self.serial.get(ctx)?;
        let service_cookie = self.service_cookie.get(ctx)?.into();

        let value = SerializedValue::serialize(&self.value)
            .with_context(|| anyhow!("failed to serialize value"))?;

        Ok(message::CallFunction2 {
            serial,
            service_cookie,
            function: self.function,
            version: self.version,
            value,
        })
    }

    pub(crate) fn matches(&self, other: &Self, ctx: &Context) -> Result<bool> {
        let res = self.serial.matches(&other.serial, ctx)?
            && self.service_cookie.matches(&other.service_cookie, ctx)?
            && (self.function == other.function)
            && (self.version == other.version)
            && self.value.matches(&other.value);

        Ok(res)
    }

    pub(crate) fn update_context(&self, other: &Self, ctx: &mut Context) -> Result<()> {
        self.serial.update_context(&other.serial, ctx)?;
        self.service_cookie
            .update_context(&other.service_cookie, ctx)?;

        Ok(())
    }

    pub(crate) fn apply_context(&self, ctx: &Context) -> Result<Self> {
        let serial = self.serial.apply_context(ctx)?;
        let service_cookie = self.service_cookie.apply_context(ctx)?;

        Ok(Self {
            serial,
            service_cookie,
            function: self.function,
            version: self.version,
            value: self.value.clone(),
        })
    }
}

impl TryFrom<message::CallFunction2> for CallFunction2 {
    type Error = Error;

    fn try_from(msg: message::CallFunction2) -> Result<Self> {
        let value = msg
            .value
            .deserialize()
            .with_context(|| anyhow!("failed to deserialize value `{:?}`", msg.value))?;

        Ok(Self {
            serial: msg.serial.into(),
            service_cookie: msg.service_cookie.into(),
            function: msg.function,
            version: msg.version,
            value,
        })
    }
}
