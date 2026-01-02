use crate::context::Context;
use aldrin_core::SerializedValue;
use aldrin_core::message::{self, ConnectReplyData};
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct ConnectReply2 {
    #[serde(flatten)]
    pub result: ConnectResult,
}

impl ConnectReply2 {
    pub(crate) fn to_core(&self, _ctx: &Context) -> message::ConnectReply2 {
        message::ConnectReply2 {
            result: self.result.into(),
            value: SerializedValue::serialize(ConnectReplyData::new()).unwrap(),
        }
    }

    pub(crate) fn matches(&self, other: &Self, _ctx: &Context) -> bool {
        self.result == other.result
    }

    pub(crate) fn apply_context(&self, _ctx: &Context) -> Self {
        self.clone()
    }
}

impl TryFrom<message::ConnectReply2> for ConnectReply2 {
    type Error = Error;

    fn try_from(msg: message::ConnectReply2) -> Result<Self> {
        Ok(Self {
            result: msg.result.into(),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "result")]
pub(crate) enum ConnectResult {
    #[serde(rename_all = "kebab-case")]
    Ok {
        minor_version: u32,
    },

    Rejected,
    IncompatibleVersion,
}

impl From<ConnectResult> for message::ConnectResult {
    fn from(res: ConnectResult) -> Self {
        match res {
            ConnectResult::Ok { minor_version } => Self::Ok(minor_version),
            ConnectResult::Rejected => Self::Rejected,
            ConnectResult::IncompatibleVersion => Self::IncompatibleVersion,
        }
    }
}

impl From<message::ConnectResult> for ConnectResult {
    fn from(res: message::ConnectResult) -> Self {
        match res {
            message::ConnectResult::Ok(minor_version) => Self::Ok { minor_version },
            message::ConnectResult::Rejected => Self::Rejected,
            message::ConnectResult::IncompatibleVersion => Self::IncompatibleVersion,
        }
    }
}
