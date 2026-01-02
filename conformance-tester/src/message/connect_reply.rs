use crate::context::Context;
use crate::value::Value;
use aldrin_core::{SerializedValue, message};
use anyhow::{Context as _, Error, Result, anyhow};
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

    pub(crate) fn matches(&self, other: &Self, _ctx: &Context) -> bool {
        match (self, other) {
            (Self::Ok, Self::Ok) => true,

            (
                Self::IncompatibleVersion { version: version1 },
                Self::IncompatibleVersion { version: version2 },
            ) => version1 == version2,

            (Self::Rejected { value: value1 }, Self::Rejected { value: value2 }) => {
                value1.matches(value2)
            }

            _ => false,
        }
    }

    pub(crate) fn apply_context(&self, _ctx: &Context) -> Self {
        self.clone()
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
