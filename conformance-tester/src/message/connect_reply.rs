use crate::context::Context;
use crate::value::Value;
use aldrin_core::{message, SerializedValue};
use anyhow::{anyhow, Context as _, Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "result")]
pub(crate) enum ConnectReply {
    Ok,
    IncompatibleVersion { version: u32 },
    Rejected { value: Value },
}

impl ConnectReply {
    pub(crate) fn to_core(&self, _ctx: &Context) -> Result<message::ConnectReply> {
        match self {
            Self::Ok => {
                let value = SerializedValue::serialize(())
                    .with_context(|| anyhow!("failed to serialize value"))?;

                Ok(message::ConnectReply::Ok(value))
            }

            Self::IncompatibleVersion { version } => {
                Ok(message::ConnectReply::IncompatibleVersion(*version))
            }

            Self::Rejected { value } => {
                let value = SerializedValue::serialize(value)
                    .with_context(|| anyhow!("failed to serialize value"))?;

                Ok(message::ConnectReply::Rejected(value))
            }
        }
    }

    pub(crate) fn matches(&self, other: &Self, _ctx: &Context) -> Result<bool> {
        match (self, other) {
            (Self::Ok, Self::Ok) => Ok(true),

            (
                Self::IncompatibleVersion { version: version1 },
                Self::IncompatibleVersion { version: version2 },
            ) => Ok(version1 == version2),

            (Self::Rejected { value: value1 }, Self::Rejected { value: value2 }) => {
                Ok(value1.matches(value2))
            }

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

impl TryFrom<message::ConnectReply> for ConnectReply {
    type Error = Error;

    fn try_from(msg: message::ConnectReply) -> Result<Self> {
        match msg {
            message::ConnectReply::Ok(_) => Ok(Self::Ok),

            message::ConnectReply::IncompatibleVersion(version) => {
                Ok(Self::IncompatibleVersion { version })
            }

            message::ConnectReply::Rejected(value) => {
                let value = value
                    .deserialize()
                    .with_context(|| anyhow!("failed to deserialize value `{:?}`", value))?;

                Ok(Self::Rejected { value })
            }
        }
    }
}
