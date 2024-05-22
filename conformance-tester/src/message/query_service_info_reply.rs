use super::ServiceInfo;
use crate::context::Context;
use crate::serial::Serial;
use aldrin_core::{message, ServiceInfo as CoreServiceInfo};
use anyhow::{anyhow, Context as _, Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct QueryServiceInfoReply {
    pub serial: Serial,

    #[serde(flatten)]
    pub result: QueryServiceInfoResult,
}

impl QueryServiceInfoReply {
    pub fn to_core(&self, ctx: &Context) -> Result<message::QueryServiceInfoReply> {
        let serial = self.serial.get(ctx)?;
        let result = self.result.to_core(ctx)?;

        Ok(message::QueryServiceInfoReply { serial, result })
    }

    pub fn matches(&self, other: &Self, ctx: &Context) -> Result<bool> {
        let res =
            self.serial.matches(&other.serial, ctx)? && self.result.matches(&other.result, ctx)?;

        Ok(res)
    }

    pub fn update_context(&self, other: &Self, ctx: &mut Context) -> Result<()> {
        self.serial.update_context(&other.serial, ctx)?;
        self.result.update_context(&other.result, ctx)?;

        Ok(())
    }

    pub fn apply_context(&self, ctx: &Context) -> Result<Self> {
        let serial = self.serial.apply_context(ctx)?;
        let result = self.result.apply_context(ctx)?;

        Ok(Self { serial, result })
    }
}

impl TryFrom<message::QueryServiceInfoReply> for QueryServiceInfoReply {
    type Error = Error;

    fn try_from(msg: message::QueryServiceInfoReply) -> Result<Self> {
        let result = msg.result.try_into()?;

        Ok(Self {
            serial: msg.serial.into(),
            result,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "result")]
pub enum QueryServiceInfoResult {
    Ok {
        #[serde(flatten)]
        info: ServiceInfo,
    },

    InvalidService,
}

impl QueryServiceInfoResult {
    pub fn to_core(&self, ctx: &Context) -> Result<message::QueryServiceInfoResult> {
        match self {
            Self::Ok { info } => info.to_core(ctx).and_then(|info| {
                message::QueryServiceInfoResult::ok_with_serialize_info(info)
                    .with_context(|| anyhow!("failed to serialize value"))
            }),

            Self::InvalidService => Ok(message::QueryServiceInfoResult::InvalidService),
        }
    }

    pub fn matches(&self, other: &Self, ctx: &Context) -> Result<bool> {
        match (self, other) {
            (Self::Ok { info: i1 }, Self::Ok { info: i2 }) => i1.matches(i2, ctx),
            (Self::InvalidService, Self::InvalidService) => Ok(true),
            _ => Ok(false),
        }
    }

    pub fn update_context(&self, other: &Self, ctx: &mut Context) -> Result<()> {
        if let (Self::Ok { info: i1 }, Self::Ok { info: i2 }) = (self, other) {
            i1.update_context(i2, ctx)
        } else {
            Ok(())
        }
    }

    pub fn apply_context(&self, ctx: &Context) -> Result<Self> {
        match self {
            Self::Ok { info } => {
                let info = info.apply_context(ctx)?;
                Ok(Self::Ok { info })
            }

            Self::InvalidService => Ok(Self::InvalidService),
        }
    }
}

impl TryFrom<message::QueryServiceInfoResult> for QueryServiceInfoResult {
    type Error = Error;

    fn try_from(res: message::QueryServiceInfoResult) -> Result<Self> {
        match res {
            message::QueryServiceInfoResult::Ok(value) => {
                let info = value
                    .deserialize::<CoreServiceInfo>()
                    .with_context(|| anyhow!("failed to deserialize value `{:?}`", value))?
                    .into();

                Ok(Self::Ok { info })
            }

            message::QueryServiceInfoResult::InvalidService => Ok(Self::InvalidService),
        }
    }
}
