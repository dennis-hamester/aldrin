mod call_function;
mod call_function_reply;
mod connect;
mod connect_reply;
mod create_object;
mod create_object_reply;
mod create_service;
mod create_service_reply;
mod destroy_object;
mod destroy_object_reply;
mod destroy_service;
mod destroy_service_reply;
mod object_created_event;
mod object_destroyed_event;
mod service_created_event;
mod service_destroyed_event;
mod shutdown;
mod subscribe_event;
mod subscribe_event_reply;
mod subscribe_objects;
mod subscribe_objects_reply;
mod subscribe_services;
mod subscribe_services_reply;
mod sync;
mod sync_reply;
mod unsubscribe_event;
mod unsubscribe_objects;
mod unsubscribe_services;

use crate::context::Context;
use aldrin_proto::message::Message as ProtoMessage;
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};
use std::fmt;

pub use call_function::CallFunction;
pub use call_function_reply::{CallFunctionReply, CallFunctionResult};
pub use connect::Connect;
pub use connect_reply::ConnectReply;
pub use create_object::CreateObject;
pub use create_object_reply::{CreateObjectReply, CreateObjectResult};
pub use create_service::CreateService;
pub use create_service_reply::{CreateServiceReply, CreateServiceResult};
pub use destroy_object::DestroyObject;
pub use destroy_object_reply::{DestroyObjectReply, DestroyObjectResult};
pub use destroy_service::DestroyService;
pub use destroy_service_reply::{DestroyServiceReply, DestroyServiceResult};
pub use object_created_event::ObjectCreatedEvent;
pub use object_destroyed_event::ObjectDestroyedEvent;
pub use service_created_event::ServiceCreatedEvent;
pub use service_destroyed_event::ServiceDestroyedEvent;
pub use shutdown::Shutdown;
pub use subscribe_event::SubscribeEvent;
pub use subscribe_event_reply::{SubscribeEventReply, SubscribeEventResult};
pub use subscribe_objects::SubscribeObjects;
pub use subscribe_objects_reply::SubscribeObjectsReply;
pub use subscribe_services::SubscribeServices;
pub use subscribe_services_reply::SubscribeServicesReply;
pub use sync::Sync;
pub use sync_reply::SyncReply;
pub use unsubscribe_event::UnsubscribeEvent;
pub use unsubscribe_objects::UnsubscribeObjects;
pub use unsubscribe_services::UnsubscribeServices;

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
    SubscribeObjectsReply(SubscribeObjectsReply),
    UnsubscribeObjects(UnsubscribeObjects),
    ObjectCreatedEvent(ObjectCreatedEvent),
    ObjectDestroyedEvent(ObjectDestroyedEvent),
    CreateService(CreateService),
    CreateServiceReply(CreateServiceReply),
    DestroyService(DestroyService),
    DestroyServiceReply(DestroyServiceReply),
    SubscribeServices(SubscribeServices),
    SubscribeServicesReply(SubscribeServicesReply),
    UnsubscribeServices(UnsubscribeServices),
    ServiceCreatedEvent(ServiceCreatedEvent),
    ServiceDestroyedEvent(ServiceDestroyedEvent),
    CallFunction(CallFunction),
    CallFunctionReply(CallFunctionReply),
    SubscribeEvent(SubscribeEvent),
    SubscribeEventReply(SubscribeEventReply),
    UnsubscribeEvent(UnsubscribeEvent),
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
            Self::SubscribeObjectsReply(msg) => {
                msg.to_proto(ctx).map(ProtoMessage::SubscribeObjectsReply)
            }
            Self::UnsubscribeObjects(msg) => {
                msg.to_proto(ctx).map(ProtoMessage::UnsubscribeObjects)
            }
            Self::ObjectCreatedEvent(msg) => {
                msg.to_proto(ctx).map(ProtoMessage::ObjectCreatedEvent)
            }
            Self::ObjectDestroyedEvent(msg) => {
                msg.to_proto(ctx).map(ProtoMessage::ObjectDestroyedEvent)
            }
            Self::CreateService(msg) => msg.to_proto(ctx).map(ProtoMessage::CreateService),
            Self::CreateServiceReply(msg) => {
                msg.to_proto(ctx).map(ProtoMessage::CreateServiceReply)
            }
            Self::DestroyService(msg) => msg.to_proto(ctx).map(ProtoMessage::DestroyService),
            Self::DestroyServiceReply(msg) => {
                msg.to_proto(ctx).map(ProtoMessage::DestroyServiceReply)
            }
            Self::SubscribeServices(msg) => msg.to_proto(ctx).map(ProtoMessage::SubscribeServices),
            Self::SubscribeServicesReply(msg) => {
                msg.to_proto(ctx).map(ProtoMessage::SubscribeServicesReply)
            }
            Self::UnsubscribeServices(msg) => {
                msg.to_proto(ctx).map(ProtoMessage::UnsubscribeServices)
            }
            Self::ServiceCreatedEvent(msg) => {
                msg.to_proto(ctx).map(ProtoMessage::ServiceCreatedEvent)
            }
            Self::ServiceDestroyedEvent(msg) => {
                msg.to_proto(ctx).map(ProtoMessage::ServiceDestroyedEvent)
            }
            Self::CallFunction(msg) => msg.to_proto(ctx).map(ProtoMessage::CallFunction),
            Self::CallFunctionReply(msg) => msg.to_proto(ctx).map(ProtoMessage::CallFunctionReply),
            Self::SubscribeEvent(msg) => msg.to_proto(ctx).map(ProtoMessage::SubscribeEvent),
            Self::SubscribeEventReply(msg) => {
                msg.to_proto(ctx).map(ProtoMessage::SubscribeEventReply)
            }
            Self::UnsubscribeEvent(msg) => msg.to_proto(ctx).map(ProtoMessage::UnsubscribeEvent),
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
            (Self::SubscribeObjectsReply(msg), Self::SubscribeObjectsReply(other)) => {
                msg.matches(other, ctx)
            }
            (Self::UnsubscribeObjects(msg), Self::UnsubscribeObjects(other)) => {
                msg.matches(other, ctx)
            }
            (Self::ObjectCreatedEvent(msg), Self::ObjectCreatedEvent(other)) => {
                msg.matches(other, ctx)
            }
            (Self::ObjectDestroyedEvent(msg), Self::ObjectDestroyedEvent(other)) => {
                msg.matches(other, ctx)
            }
            (Self::CreateService(msg), Self::CreateService(other)) => msg.matches(other, ctx),
            (Self::CreateServiceReply(msg), Self::CreateServiceReply(other)) => {
                msg.matches(other, ctx)
            }
            (Self::DestroyService(msg), Self::DestroyService(other)) => msg.matches(other, ctx),
            (Self::DestroyServiceReply(msg), Self::DestroyServiceReply(other)) => {
                msg.matches(other, ctx)
            }
            (Self::SubscribeServices(msg), Self::SubscribeServices(other)) => {
                msg.matches(other, ctx)
            }
            (Self::SubscribeServicesReply(msg), Self::SubscribeServicesReply(other)) => {
                msg.matches(other, ctx)
            }
            (Self::UnsubscribeServices(msg), Self::UnsubscribeServices(other)) => {
                msg.matches(other, ctx)
            }
            (Self::ServiceCreatedEvent(msg), Self::ServiceCreatedEvent(other)) => {
                msg.matches(other, ctx)
            }
            (Self::ServiceDestroyedEvent(msg), Self::ServiceDestroyedEvent(other)) => {
                msg.matches(other, ctx)
            }
            (Self::CallFunction(msg), Self::CallFunction(other)) => msg.matches(other, ctx),
            (Self::CallFunctionReply(msg), Self::CallFunctionReply(other)) => {
                msg.matches(other, ctx)
            }
            (Self::SubscribeEvent(msg), Self::SubscribeEvent(other)) => msg.matches(other, ctx),
            (Self::SubscribeEventReply(msg), Self::SubscribeEventReply(other)) => {
                msg.matches(other, ctx)
            }
            (Self::UnsubscribeEvent(msg), Self::UnsubscribeEvent(other)) => msg.matches(other, ctx),
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
            (Self::SubscribeObjectsReply(msg), Self::SubscribeObjectsReply(other)) => {
                msg.update_context(other, ctx)
            }
            (Self::UnsubscribeObjects(msg), Self::UnsubscribeObjects(other)) => {
                msg.update_context(other, ctx)
            }
            (Self::ObjectCreatedEvent(msg), Self::ObjectCreatedEvent(other)) => {
                msg.update_context(other, ctx)
            }
            (Self::ObjectDestroyedEvent(msg), Self::ObjectDestroyedEvent(other)) => {
                msg.update_context(other, ctx)
            }
            (Self::CreateService(msg), Self::CreateService(other)) => {
                msg.update_context(other, ctx)
            }
            (Self::CreateServiceReply(msg), Self::CreateServiceReply(other)) => {
                msg.update_context(other, ctx)
            }
            (Self::DestroyService(msg), Self::DestroyService(other)) => {
                msg.update_context(other, ctx)
            }
            (Self::DestroyServiceReply(msg), Self::DestroyServiceReply(other)) => {
                msg.update_context(other, ctx)
            }
            (Self::SubscribeServices(msg), Self::SubscribeServices(other)) => {
                msg.update_context(other, ctx)
            }
            (Self::SubscribeServicesReply(msg), Self::SubscribeServicesReply(other)) => {
                msg.update_context(other, ctx)
            }
            (Self::UnsubscribeServices(msg), Self::UnsubscribeServices(other)) => {
                msg.update_context(other, ctx)
            }
            (Self::ServiceCreatedEvent(msg), Self::ServiceCreatedEvent(other)) => {
                msg.update_context(other, ctx)
            }
            (Self::ServiceDestroyedEvent(msg), Self::ServiceDestroyedEvent(other)) => {
                msg.update_context(other, ctx)
            }
            (Self::CallFunction(msg), Self::CallFunction(other)) => msg.update_context(other, ctx),
            (Self::CallFunctionReply(msg), Self::CallFunctionReply(other)) => {
                msg.update_context(other, ctx)
            }
            (Self::SubscribeEvent(msg), Self::SubscribeEvent(other)) => {
                msg.update_context(other, ctx)
            }
            (Self::SubscribeEventReply(msg), Self::SubscribeEventReply(other)) => {
                msg.update_context(other, ctx)
            }
            (Self::UnsubscribeEvent(msg), Self::UnsubscribeEvent(other)) => {
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
            Self::SubscribeObjectsReply(msg) => {
                msg.apply_context(ctx).map(Self::SubscribeObjectsReply)
            }
            Self::UnsubscribeObjects(msg) => msg.apply_context(ctx).map(Self::UnsubscribeObjects),
            Self::ObjectCreatedEvent(msg) => msg.apply_context(ctx).map(Self::ObjectCreatedEvent),
            Self::ObjectDestroyedEvent(msg) => {
                msg.apply_context(ctx).map(Self::ObjectDestroyedEvent)
            }
            Self::CreateService(msg) => msg.apply_context(ctx).map(Self::CreateService),
            Self::CreateServiceReply(msg) => msg.apply_context(ctx).map(Self::CreateServiceReply),
            Self::DestroyService(msg) => msg.apply_context(ctx).map(Self::DestroyService),
            Self::DestroyServiceReply(msg) => msg.apply_context(ctx).map(Self::DestroyServiceReply),
            Self::SubscribeServices(msg) => msg.apply_context(ctx).map(Self::SubscribeServices),
            Self::SubscribeServicesReply(msg) => {
                msg.apply_context(ctx).map(Self::SubscribeServicesReply)
            }
            Self::UnsubscribeServices(msg) => msg.apply_context(ctx).map(Self::UnsubscribeServices),
            Self::ServiceCreatedEvent(msg) => msg.apply_context(ctx).map(Self::ServiceCreatedEvent),
            Self::ServiceDestroyedEvent(msg) => {
                msg.apply_context(ctx).map(Self::ServiceDestroyedEvent)
            }
            Self::CallFunction(msg) => msg.apply_context(ctx).map(Self::CallFunction),
            Self::CallFunctionReply(msg) => msg.apply_context(ctx).map(Self::CallFunctionReply),
            Self::SubscribeEvent(msg) => msg.apply_context(ctx).map(Self::SubscribeEvent),
            Self::SubscribeEventReply(msg) => msg.apply_context(ctx).map(Self::SubscribeEventReply),
            Self::UnsubscribeEvent(msg) => msg.apply_context(ctx).map(Self::UnsubscribeEvent),
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
            ProtoMessage::SubscribeObjectsReply(msg) => {
                msg.try_into().map(Self::SubscribeObjectsReply)
            }
            ProtoMessage::UnsubscribeObjects(msg) => msg.try_into().map(Self::UnsubscribeObjects),
            ProtoMessage::ObjectCreatedEvent(msg) => msg.try_into().map(Self::ObjectCreatedEvent),
            ProtoMessage::ObjectDestroyedEvent(msg) => {
                msg.try_into().map(Self::ObjectDestroyedEvent)
            }
            ProtoMessage::CreateService(msg) => msg.try_into().map(Self::CreateService),
            ProtoMessage::CreateServiceReply(msg) => msg.try_into().map(Self::CreateServiceReply),
            ProtoMessage::DestroyService(msg) => msg.try_into().map(Self::DestroyService),
            ProtoMessage::DestroyServiceReply(msg) => msg.try_into().map(Self::DestroyServiceReply),
            ProtoMessage::SubscribeServices(msg) => msg.try_into().map(Self::SubscribeServices),
            ProtoMessage::SubscribeServicesReply(msg) => {
                msg.try_into().map(Self::SubscribeServicesReply)
            }
            ProtoMessage::UnsubscribeServices(msg) => msg.try_into().map(Self::UnsubscribeServices),
            ProtoMessage::ServiceCreatedEvent(msg) => msg.try_into().map(Self::ServiceCreatedEvent),
            ProtoMessage::ServiceDestroyedEvent(msg) => {
                msg.try_into().map(Self::ServiceDestroyedEvent)
            }
            ProtoMessage::CallFunction(msg) => msg.try_into().map(Self::CallFunction),
            ProtoMessage::CallFunctionReply(msg) => msg.try_into().map(Self::CallFunctionReply),
            ProtoMessage::SubscribeEvent(msg) => msg.try_into().map(Self::SubscribeEvent),
            ProtoMessage::SubscribeEventReply(msg) => msg.try_into().map(Self::SubscribeEventReply),
            ProtoMessage::UnsubscribeEvent(msg) => msg.try_into().map(Self::UnsubscribeEvent),
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
