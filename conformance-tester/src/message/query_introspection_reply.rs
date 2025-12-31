use crate::context::Context;
use crate::serial::Serial;
use crate::value::Value;
use aldrin_core::{SerializedValue, message};
use anyhow::{Context as _, Error, Result, anyhow};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct QueryIntrospectionReply {
    pub serial: Serial,

    #[serde(flatten)]
    pub result: QueryIntrospectionResult,
}

impl QueryIntrospectionReply {
    pub(crate) fn to_core(&self, ctx: &Context) -> Result<message::QueryIntrospectionReply> {
        let serial = self.serial.get(ctx)?;
        let result = self.result.to_core(ctx)?;

        Ok(message::QueryIntrospectionReply { serial, result })
    }

    pub(crate) fn matches(&self, other: &Self, ctx: &Context) -> Result<bool> {
        let res =
            self.serial.matches(&other.serial, ctx)? && self.result.matches(&other.result, ctx)?;

        Ok(res)
    }

    pub(crate) fn update_context(&self, other: &Self, ctx: &mut Context) -> Result<()> {
        self.serial.update_context(&other.serial, ctx)?;
        self.result.update_context(&other.result, ctx)?;

        Ok(())
    }

    pub(crate) fn apply_context(&self, ctx: &Context) -> Result<Self> {
        let serial = self.serial.apply_context(ctx)?;
        let result = self.result.apply_context(ctx)?;

        Ok(Self { serial, result })
    }
}

impl TryFrom<message::QueryIntrospectionReply> for QueryIntrospectionReply {
    type Error = Error;

    fn try_from(msg: message::QueryIntrospectionReply) -> Result<Self> {
        let result = msg.result.try_into()?;

        Ok(Self {
            serial: msg.serial.into(),
            result,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "result")]
pub(crate) enum QueryIntrospectionResult {
    Ok {
        #[serde(flatten)]
        value: Value,
    },

    Unavailable,
}

impl QueryIntrospectionResult {
    pub(crate) fn to_core(&self, _ctx: &Context) -> Result<message::QueryIntrospectionResult> {
        match self {
            Self::Ok { value } => SerializedValue::serialize(value)
                .map(message::QueryIntrospectionResult::Ok)
                .with_context(|| anyhow!("failed to serialize value")),

            Self::Unavailable => Ok(message::QueryIntrospectionResult::Unavailable),
        }
    }

    pub(crate) fn matches(&self, other: &Self, _ctx: &Context) -> Result<bool> {
        match (self, other) {
            (Self::Ok { value: v1 }, Self::Ok { value: v2 }) => Ok(v1.matches(v2)),
            (Self::Unavailable, Self::Unavailable) => Ok(true),
            _ => Ok(false),
        }
    }

    pub(crate) fn update_context(&self, _other: &Self, _ctx: &mut Context) -> Result<()> {
        Ok(())
    }

    pub(crate) fn apply_context(&self, _ctx: &Context) -> Result<Self> {
        Ok(self.clone())
    }
}

impl TryFrom<message::QueryIntrospectionResult> for QueryIntrospectionResult {
    type Error = Error;

    fn try_from(res: message::QueryIntrospectionResult) -> Result<Self> {
        match res {
            message::QueryIntrospectionResult::Ok(value) => {
                let value = value
                    .deserialize()
                    .with_context(|| anyhow!("failed to deserialize value `{:?}`", value))?;

                Ok(Self::Ok { value })
            }

            message::QueryIntrospectionResult::Unavailable => Ok(Self::Unavailable),
        }
    }
}
