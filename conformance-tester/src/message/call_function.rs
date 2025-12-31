use crate::context::Context;
use crate::serial::Serial;
use crate::uuid_ref::UuidRef;
use crate::value::Value;
use aldrin_core::{SerializedValue, message};
use anyhow::{Context as _, Error, Result, anyhow};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct CallFunction {
    pub serial: Serial,
    pub function: u32,
    pub service_cookie: UuidRef,

    #[serde(flatten)]
    pub value: Value,
}

impl CallFunction {
    pub(crate) fn to_core(&self, ctx: &Context) -> Result<message::CallFunction> {
        let serial = self.serial.get(ctx)?;
        let service_cookie = self.service_cookie.get(ctx)?.into();

        let value = SerializedValue::serialize(&self.value)
            .with_context(|| anyhow!("failed to serialize value"))?;

        Ok(message::CallFunction {
            serial,
            service_cookie,
            function: self.function,
            value,
        })
    }

    pub(crate) fn matches(&self, other: &Self, ctx: &Context) -> Result<bool> {
        let res = self.serial.matches(&other.serial, ctx)?
            && (self.function == other.function)
            && self.service_cookie.matches(&other.service_cookie, ctx)?
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
            function: self.function,
            service_cookie,
            value: self.value.clone(),
        })
    }
}

impl TryFrom<message::CallFunction> for CallFunction {
    type Error = Error;

    fn try_from(msg: message::CallFunction) -> Result<Self> {
        let value = msg
            .value
            .deserialize()
            .with_context(|| anyhow!("failed to deserialize value `{:?}`", msg.value))?;

        Ok(Self {
            serial: msg.serial.into(),
            function: msg.function,
            service_cookie: msg.service_cookie.into(),
            value,
        })
    }
}
