mod connect;
mod connect_reply;
mod create_object;
mod create_object_reply;
mod destroy_object;
mod destroy_object_reply;
mod shutdown;
mod subscribe_objects;
mod sync;
mod sync_reply;

use crate::context::Context;
use aldrin_proto::message::Message as ProtoMessage;
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};
use std::fmt;

pub use connect::Connect;
pub use connect_reply::ConnectReply;
pub use create_object::CreateObject;
pub use create_object_reply::{CreateObjectReply, CreateObjectResult};
pub use destroy_object::DestroyObject;
pub use destroy_object_reply::{DestroyObjectReply, DestroyObjectResult};
pub use shutdown::Shutdown;
pub use subscribe_objects::SubscribeObjects;
pub use sync::Sync;
pub use sync_reply::SyncReply;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "message")]
pub enum Message {
    Connect(Connect),
    ConnectReply(ConnectReply),
    Shutdown(Shutdown),
    CreateObject(CreateObject),
    CreateObjectReply(CreateObjectReply),
    DestroyObject(DestroyObject),
    DestroyObjectReply(DestroyObjectReply),
    SubscribeObjects(SubscribeObjects),
    Sync(Sync),
    SyncReply(SyncReply),
}

impl Message {
    pub fn to_proto(&self, ctx: &Context) -> Result<ProtoMessage> {
        match self {
            Self::Connect(msg) => msg.to_proto(ctx).map(ProtoMessage::Connect),
            Self::ConnectReply(msg) => msg.to_proto(ctx).map(ProtoMessage::ConnectReply),
            Self::Shutdown(msg) => msg.to_proto(ctx).map(ProtoMessage::Shutdown),
            Self::CreateObject(msg) => msg.to_proto(ctx).map(ProtoMessage::CreateObject),
            Self::CreateObjectReply(msg) => msg.to_proto(ctx).map(ProtoMessage::CreateObjectReply),
            Self::DestroyObject(msg) => msg.to_proto(ctx).map(ProtoMessage::DestroyObject),
            Self::DestroyObjectReply(msg) => {
                msg.to_proto(ctx).map(ProtoMessage::DestroyObjectReply)
            }
            Self::SubscribeObjects(msg) => msg.to_proto(ctx).map(ProtoMessage::SubscribeObjects),
            Self::Sync(msg) => msg.to_proto(ctx).map(ProtoMessage::Sync),
            Self::SyncReply(msg) => msg.to_proto(ctx).map(ProtoMessage::SyncReply),
        }
    }

    pub fn matches(&self, other: &Message, ctx: &Context) -> Result<bool> {
        match (self, other) {
            (Self::Connect(msg), Self::Connect(other)) => msg.matches(other, ctx),
            (Self::ConnectReply(msg), Self::ConnectReply(other)) => msg.matches(other, ctx),
            (Self::Shutdown(msg), Self::Shutdown(other)) => msg.matches(other, ctx),
            (Self::CreateObject(msg), Self::CreateObject(other)) => msg.matches(other, ctx),
            (Self::CreateObjectReply(msg), Self::CreateObjectReply(other)) => {
                msg.matches(other, ctx)
            }
            (Self::DestroyObject(msg), Self::DestroyObject(other)) => msg.matches(other, ctx),
            (Self::DestroyObjectReply(msg), Self::DestroyObjectReply(other)) => {
                msg.matches(other, ctx)
            }
            (Self::SubscribeObjects(msg), Self::SubscribeObjects(other)) => msg.matches(other, ctx),
            (Self::Sync(msg), Self::Sync(other)) => msg.matches(other, ctx),
            (Self::SyncReply(msg), Self::SyncReply(other)) => msg.matches(other, ctx),
            _ => Ok(false),
        }
    }

    pub fn update_context(&self, other: &Message, ctx: &mut Context) -> Result<()> {
        match (self, other) {
            (Self::Connect(msg), Self::Connect(other)) => msg.update_context(other, ctx),
            (Self::ConnectReply(msg), Self::ConnectReply(other)) => msg.update_context(other, ctx),
            (Self::Shutdown(msg), Self::Shutdown(other)) => msg.update_context(other, ctx),
            (Self::CreateObject(msg), Self::CreateObject(other)) => msg.update_context(other, ctx),
            (Self::CreateObjectReply(msg), Self::CreateObjectReply(other)) => {
                msg.update_context(other, ctx)
            }
            (Self::DestroyObject(msg), Self::DestroyObject(other)) => {
                msg.update_context(other, ctx)
            }
            (Self::DestroyObjectReply(msg), Self::DestroyObjectReply(other)) => {
                msg.update_context(other, ctx)
            }
            (Self::SubscribeObjects(msg), Self::SubscribeObjects(other)) => {
                msg.update_context(other, ctx)
            }
            (Self::Sync(msg), Self::Sync(other)) => msg.update_context(other, ctx),
            (Self::SyncReply(msg), Self::SyncReply(other)) => msg.update_context(other, ctx),
            _ => unreachable!(),
        }
    }

    pub fn apply_context(&self, ctx: &Context) -> Result<Self> {
        match self {
            Self::Connect(msg) => msg.apply_context(ctx).map(Self::Connect),
            Self::ConnectReply(msg) => msg.apply_context(ctx).map(Self::ConnectReply),
            Self::Shutdown(msg) => msg.apply_context(ctx).map(Self::Shutdown),
            Self::CreateObject(msg) => msg.apply_context(ctx).map(Self::CreateObject),
            Self::CreateObjectReply(msg) => msg.apply_context(ctx).map(Self::CreateObjectReply),
            Self::DestroyObject(msg) => msg.apply_context(ctx).map(Self::DestroyObject),
            Self::DestroyObjectReply(msg) => msg.apply_context(ctx).map(Self::DestroyObjectReply),
            Self::SubscribeObjects(msg) => msg.apply_context(ctx).map(Self::SubscribeObjects),
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
            ProtoMessage::Shutdown(msg) => msg.try_into().map(Self::Shutdown),
            ProtoMessage::CreateObject(msg) => msg.try_into().map(Self::CreateObject),
            ProtoMessage::CreateObjectReply(msg) => msg.try_into().map(Self::CreateObjectReply),
            ProtoMessage::DestroyObject(msg) => msg.try_into().map(Self::DestroyObject),
            ProtoMessage::DestroyObjectReply(msg) => msg.try_into().map(Self::DestroyObjectReply),
            ProtoMessage::SubscribeObjects(msg) => msg.try_into().map(Self::SubscribeObjects),
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
