mod connect;
mod connect_reply;
mod sync;
mod sync_reply;

use crate::context::Context;
use aldrin_proto::message::Message as ProtoMessage;
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};
use std::fmt;

pub use connect::Connect;
pub use connect_reply::ConnectReply;
pub use sync::Sync;
pub use sync_reply::SyncReply;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "message")]
pub enum Message {
    Connect(Connect),
    ConnectReply(ConnectReply),
    Sync(Sync),
    SyncReply(SyncReply),
}

impl Message {
    pub fn to_proto(&self, ctx: &Context) -> Result<ProtoMessage> {
        match self {
            Self::Connect(msg) => msg.to_proto(ctx).map(ProtoMessage::Connect),
            Self::ConnectReply(msg) => msg.to_proto(ctx).map(ProtoMessage::ConnectReply),
            Self::Sync(msg) => msg.to_proto(ctx).map(ProtoMessage::Sync),
            Self::SyncReply(msg) => msg.to_proto(ctx).map(ProtoMessage::SyncReply),
        }
    }

    pub fn matches(&self, other: &Message, ctx: &Context) -> Result<bool> {
        match (self, other) {
            (Self::Connect(msg), Self::Connect(other)) => msg.matches(other, ctx),
            (Self::ConnectReply(msg), Self::ConnectReply(other)) => msg.matches(other, ctx),
            (Self::Sync(msg), Self::Sync(other)) => msg.matches(other, ctx),
            (Self::SyncReply(msg), Self::SyncReply(other)) => msg.matches(other, ctx),
            _ => Ok(false),
        }
    }

    pub fn update_context(&self, other: &Message, ctx: &mut Context) -> Result<()> {
        match (self, other) {
            (Self::Connect(msg), Self::Connect(other)) => msg.update_context(other, ctx),
            (Self::ConnectReply(msg), Self::ConnectReply(other)) => msg.update_context(other, ctx),
            (Self::Sync(msg), Self::Sync(other)) => msg.update_context(other, ctx),
            (Self::SyncReply(msg), Self::SyncReply(other)) => msg.update_context(other, ctx),
            _ => unreachable!(),
        }
    }

    pub fn apply_context(&self, ctx: &Context) -> Result<Self> {
        match self {
            Self::Connect(msg) => msg.apply_context(ctx).map(Self::Connect),
            Self::ConnectReply(msg) => msg.apply_context(ctx).map(Self::ConnectReply),
            Self::Sync(msg) => msg.apply_context(ctx).map(Self::Sync),
            Self::SyncReply(msg) => msg.apply_context(ctx).map(Self::SyncReply),
        }
    }
}

impl TryFrom<ProtoMessage> for Message {
    type Error = Error;

    fn try_from(msg: ProtoMessage) -> Result<Self> {
        match msg {
            ProtoMessage::Connect(msg) => msg.try_into().map(Self::Connect),
            ProtoMessage::ConnectReply(msg) => msg.try_into().map(Self::ConnectReply),
            ProtoMessage::Sync(msg) => msg.try_into().map(Self::Sync),
            ProtoMessage::SyncReply(msg) => msg.try_into().map(Self::SyncReply),
            _ => todo!(),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ChannelEnd {
    Sender,
    Receiver,
}

impl From<aldrin_proto::message::ChannelEnd> for ChannelEnd {
    fn from(end: aldrin_proto::message::ChannelEnd) -> Self {
        match end {
            aldrin_proto::message::ChannelEnd::Sender => Self::Sender,
            aldrin_proto::message::ChannelEnd::Receiver => Self::Receiver,
        }
    }
}

impl From<ChannelEnd> for aldrin_proto::message::ChannelEnd {
    fn from(end: ChannelEnd) -> Self {
        match end {
            ChannelEnd::Sender => Self::Sender,
            ChannelEnd::Receiver => Self::Receiver,
        }
    }
}

impl fmt::Display for ChannelEnd {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Sender => f.pad("sender"),
            Self::Receiver => f.pad("receiver"),
        }
    }
}
