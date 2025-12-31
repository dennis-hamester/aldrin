use crate::context::Context;
use crate::serial::Serial;
use crate::value::Value;
use aldrin_core::{SerializedValue, message};
use anyhow::{Context as _, Error, Result, anyhow};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct CallFunctionReply {
    pub serial: Serial,

    #[serde(flatten)]
    pub result: CallFunctionResult,
}

impl CallFunctionReply {
    pub(crate) fn to_core(&self, ctx: &Context) -> Result<message::CallFunctionReply> {
        let serial = self.serial.get(ctx)?;
        let result = self.result.to_core(ctx)?;

        Ok(message::CallFunctionReply { serial, result })
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

impl TryFrom<message::CallFunctionReply> for CallFunctionReply {
    type Error = Error;

    fn try_from(msg: message::CallFunctionReply) -> Result<Self> {
        let result = msg.result.try_into()?;

        Ok(Self {
            serial: msg.serial.into(),
            result,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "result")]
pub(crate) enum CallFunctionResult {
    Ok {
        #[serde(flatten)]
        value: Value,
    },

    Err {
        #[serde(flatten)]
        value: Value,
    },

    Aborted,
    InvalidService,
    InvalidFunction,
    InvalidArgs,
}

impl CallFunctionResult {
    pub(crate) fn to_core(&self, _ctx: &Context) -> Result<message::CallFunctionResult> {
        match self {
            Self::Ok { value } => {
                let value = SerializedValue::serialize(value)
                    .with_context(|| anyhow!("failed to serialize value"))?;

                Ok(message::CallFunctionResult::Ok(value))
            }

            Self::Err { value } => {
                let value = SerializedValue::serialize(value)
                    .with_context(|| anyhow!("failed to serialize value"))?;

                Ok(message::CallFunctionResult::Err(value))
            }

            Self::Aborted => Ok(message::CallFunctionResult::Aborted),
            Self::InvalidService => Ok(message::CallFunctionResult::InvalidService),
            Self::InvalidFunction => Ok(message::CallFunctionResult::InvalidFunction),
            Self::InvalidArgs => Ok(message::CallFunctionResult::InvalidArgs),
        }
    }

    pub(crate) fn matches(&self, other: &Self, _ctx: &Context) -> Result<bool> {
        match (self, other) {
            (Self::Ok { value: v1 }, Self::Ok { value: v2 }) => Ok(v1.matches(v2)),
            (Self::Err { value: v1 }, Self::Err { value: v2 }) => Ok(v1.matches(v2)),

            (Self::Aborted, Self::Aborted)
            | (Self::InvalidService, Self::InvalidService)
            | (Self::InvalidFunction, Self::InvalidFunction)
            | (Self::InvalidArgs, Self::InvalidArgs) => Ok(true),

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

impl TryFrom<message::CallFunctionResult> for CallFunctionResult {
    type Error = Error;

    fn try_from(res: message::CallFunctionResult) -> Result<Self> {
        match res {
            message::CallFunctionResult::Ok(value) => {
                let value = value
                    .deserialize()
                    .with_context(|| anyhow!("failed to deserialize value `{:?}`", value))?;

                Ok(Self::Ok { value })
            }

            message::CallFunctionResult::Err(value) => {
                let value = value
                    .deserialize()
                    .with_context(|| anyhow!("failed to deserialize value `{:?}`", value))?;

                Ok(Self::Err { value })
            }

            message::CallFunctionResult::Aborted => Ok(Self::Aborted),
            message::CallFunctionResult::InvalidService => Ok(Self::InvalidService),
            message::CallFunctionResult::InvalidFunction => Ok(Self::InvalidFunction),
            message::CallFunctionResult::InvalidArgs => Ok(Self::InvalidArgs),
        }
    }
}
