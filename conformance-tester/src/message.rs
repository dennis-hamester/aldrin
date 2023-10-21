mod add_channel_capacity;
mod call_function;
mod call_function_reply;
mod channel_end_claimed;
mod channel_end_closed;
mod claim_channel_end;
mod claim_channel_end_reply;
mod close_channel_end;
mod close_channel_end_reply;
mod connect;
mod connect_reply;
mod create_channel;
mod create_channel_reply;
mod create_object;
mod create_object_reply;
mod create_service;
mod create_service_reply;
mod destroy_object;
mod destroy_object_reply;
mod destroy_service;
mod destroy_service_reply;
mod emit_event;
mod item_received;
mod object_created_event;
mod object_destroyed_event;
mod query_object;
mod query_object_reply;
mod query_service_version;
mod query_service_version_reply;
mod send_item;
mod service_created_event;
mod service_destroyed;
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

pub use add_channel_capacity::AddChannelCapacity;
pub use call_function::CallFunction;
pub use call_function_reply::CallFunctionReply;
pub use channel_end_claimed::ChannelEndClaimed;
pub use channel_end_closed::ChannelEndClosed;
pub use claim_channel_end::ClaimChannelEnd;
pub use claim_channel_end_reply::{ClaimChannelEndReply, ClaimChannelEndResult};
pub use close_channel_end::CloseChannelEnd;
pub use close_channel_end_reply::{CloseChannelEndReply, CloseChannelEndResult};
pub use connect::Connect;
pub use connect_reply::ConnectReply;
pub use create_channel::CreateChannel;
pub use create_channel_reply::CreateChannelReply;
pub use create_object::CreateObject;
pub use create_object_reply::{CreateObjectReply, CreateObjectResult};
pub use create_service::CreateService;
pub use create_service_reply::{CreateServiceReply, CreateServiceResult};
pub use destroy_object::DestroyObject;
pub use destroy_object_reply::{DestroyObjectReply, DestroyObjectResult};
pub use destroy_service::DestroyService;
pub use destroy_service_reply::{DestroyServiceReply, DestroyServiceResult};
pub use emit_event::EmitEvent;
pub use item_received::ItemReceived;
pub use object_created_event::ObjectCreatedEvent;
pub use object_destroyed_event::ObjectDestroyedEvent;
pub use query_object::QueryObject;
pub use query_object_reply::QueryObjectReply;
pub use query_service_version::QueryServiceVersion;
pub use query_service_version_reply::QueryServiceVersionReply;
pub use send_item::SendItem;
pub use service_created_event::ServiceCreatedEvent;
pub use service_destroyed::ServiceDestroyed;
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
    EmitEvent(EmitEvent),
    QueryObject(QueryObject),
    QueryObjectReply(QueryObjectReply),
    QueryServiceVersion(QueryServiceVersion),
    QueryServiceVersionReply(QueryServiceVersionReply),
    CreateChannel(CreateChannel),
    CreateChannelReply(CreateChannelReply),
    CloseChannelEnd(CloseChannelEnd),
    CloseChannelEndReply(CloseChannelEndReply),
    ChannelEndClosed(ChannelEndClosed),
    ClaimChannelEnd(ClaimChannelEnd),
    ClaimChannelEndReply(ClaimChannelEndReply),
    ChannelEndClaimed(ChannelEndClaimed),
    SendItem(SendItem),
    ItemReceived(ItemReceived),
    AddChannelCapacity(AddChannelCapacity),
    Sync(Sync),
    SyncReply(SyncReply),
    ServiceDestroyed(ServiceDestroyed),
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
            Self::EmitEvent(msg) => msg.to_proto(ctx).map(ProtoMessage::EmitEvent),
            Self::QueryObject(msg) => msg.to_proto(ctx).map(ProtoMessage::QueryObject),
            Self::QueryObjectReply(msg) => msg.to_proto(ctx).map(ProtoMessage::QueryObjectReply),
            Self::QueryServiceVersion(msg) => {
                msg.to_proto(ctx).map(ProtoMessage::QueryServiceVersion)
            }
            Self::QueryServiceVersionReply(msg) => msg
                .to_proto(ctx)
                .map(ProtoMessage::QueryServiceVersionReply),
            Self::CreateChannel(msg) => msg.to_proto(ctx).map(ProtoMessage::CreateChannel),
            Self::CreateChannelReply(msg) => {
                msg.to_proto(ctx).map(ProtoMessage::CreateChannelReply)
            }
            Self::CloseChannelEnd(msg) => msg.to_proto(ctx).map(ProtoMessage::CloseChannelEnd),
            Self::CloseChannelEndReply(msg) => {
                msg.to_proto(ctx).map(ProtoMessage::CloseChannelEndReply)
            }
            Self::ChannelEndClosed(msg) => msg.to_proto(ctx).map(ProtoMessage::ChannelEndClosed),
            Self::ClaimChannelEnd(msg) => msg.to_proto(ctx).map(ProtoMessage::ClaimChannelEnd),
            Self::ClaimChannelEndReply(msg) => {
                msg.to_proto(ctx).map(ProtoMessage::ClaimChannelEndReply)
            }
            Self::ChannelEndClaimed(msg) => msg.to_proto(ctx).map(ProtoMessage::ChannelEndClaimed),
            Self::SendItem(msg) => msg.to_proto(ctx).map(ProtoMessage::SendItem),
            Self::ItemReceived(msg) => msg.to_proto(ctx).map(ProtoMessage::ItemReceived),
            Self::AddChannelCapacity(msg) => {
                msg.to_proto(ctx).map(ProtoMessage::AddChannelCapacity)
            }
            Self::Sync(msg) => msg.to_proto(ctx).map(ProtoMessage::Sync),
            Self::SyncReply(msg) => msg.to_proto(ctx).map(ProtoMessage::SyncReply),
            Self::ServiceDestroyed(msg) => msg.to_proto(ctx).map(ProtoMessage::ServiceDestroyed),
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
            (Self::EmitEvent(msg), Self::EmitEvent(other)) => msg.matches(other, ctx),
            (Self::QueryObject(msg), Self::QueryObject(other)) => msg.matches(other, ctx),
            (Self::QueryObjectReply(msg), Self::QueryObjectReply(other)) => msg.matches(other, ctx),
            (Self::QueryServiceVersion(msg), Self::QueryServiceVersion(other)) => {
                msg.matches(other, ctx)
            }
            (Self::QueryServiceVersionReply(msg), Self::QueryServiceVersionReply(other)) => {
                msg.matches(other, ctx)
            }
            (Self::CreateChannel(msg), Self::CreateChannel(other)) => msg.matches(other, ctx),
            (Self::CreateChannelReply(msg), Self::CreateChannelReply(other)) => {
                msg.matches(other, ctx)
            }
            (Self::CloseChannelEnd(msg), Self::CloseChannelEnd(other)) => msg.matches(other, ctx),
            (Self::CloseChannelEndReply(msg), Self::CloseChannelEndReply(other)) => {
                msg.matches(other, ctx)
            }
            (Self::ChannelEndClosed(msg), Self::ChannelEndClosed(other)) => msg.matches(other, ctx),
            (Self::ClaimChannelEnd(msg), Self::ClaimChannelEnd(other)) => msg.matches(other, ctx),
            (Self::ClaimChannelEndReply(msg), Self::ClaimChannelEndReply(other)) => {
                msg.matches(other, ctx)
            }
            (Self::ChannelEndClaimed(msg), Self::ChannelEndClaimed(other)) => {
                msg.matches(other, ctx)
            }
            (Self::SendItem(msg), Self::SendItem(other)) => msg.matches(other, ctx),
            (Self::ItemReceived(msg), Self::ItemReceived(other)) => msg.matches(other, ctx),
            (Self::AddChannelCapacity(msg), Self::AddChannelCapacity(other)) => {
                msg.matches(other, ctx)
            }
            (Self::Sync(msg), Self::Sync(other)) => msg.matches(other, ctx),
            (Self::SyncReply(msg), Self::SyncReply(other)) => msg.matches(other, ctx),
            (Self::ServiceDestroyed(msg), Self::ServiceDestroyed(other)) => msg.matches(other, ctx),
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
            (Self::EmitEvent(msg), Self::EmitEvent(other)) => msg.update_context(other, ctx),
            (Self::QueryObject(msg), Self::QueryObject(other)) => msg.update_context(other, ctx),
            (Self::QueryObjectReply(msg), Self::QueryObjectReply(other)) => {
                msg.update_context(other, ctx)
            }
            (Self::QueryServiceVersion(msg), Self::QueryServiceVersion(other)) => {
                msg.update_context(other, ctx)
            }
            (Self::QueryServiceVersionReply(msg), Self::QueryServiceVersionReply(other)) => {
                msg.update_context(other, ctx)
            }
            (Self::CreateChannel(msg), Self::CreateChannel(other)) => {
                msg.update_context(other, ctx)
            }
            (Self::CreateChannelReply(msg), Self::CreateChannelReply(other)) => {
                msg.update_context(other, ctx)
            }
            (Self::CloseChannelEnd(msg), Self::CloseChannelEnd(other)) => {
                msg.update_context(other, ctx)
            }
            (Self::CloseChannelEndReply(msg), Self::CloseChannelEndReply(other)) => {
                msg.update_context(other, ctx)
            }
            (Self::ChannelEndClosed(msg), Self::ChannelEndClosed(other)) => {
                msg.update_context(other, ctx)
            }
            (Self::ClaimChannelEnd(msg), Self::ClaimChannelEnd(other)) => {
                msg.update_context(other, ctx)
            }
            (Self::ClaimChannelEndReply(msg), Self::ClaimChannelEndReply(other)) => {
                msg.update_context(other, ctx)
            }
            (Self::ChannelEndClaimed(msg), Self::ChannelEndClaimed(other)) => {
                msg.update_context(other, ctx)
            }
            (Self::SendItem(msg), Self::SendItem(other)) => msg.update_context(other, ctx),
            (Self::ItemReceived(msg), Self::ItemReceived(other)) => msg.update_context(other, ctx),
            (Self::AddChannelCapacity(msg), Self::AddChannelCapacity(other)) => {
                msg.update_context(other, ctx)
            }
            (Self::Sync(msg), Self::Sync(other)) => msg.update_context(other, ctx),
            (Self::SyncReply(msg), Self::SyncReply(other)) => msg.update_context(other, ctx),
            (Self::ServiceDestroyed(msg), Self::ServiceDestroyed(other)) => {
                msg.update_context(other, ctx)
            }
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
            Self::EmitEvent(msg) => msg.apply_context(ctx).map(Self::EmitEvent),
            Self::QueryObject(msg) => msg.apply_context(ctx).map(Self::QueryObject),
            Self::QueryObjectReply(msg) => msg.apply_context(ctx).map(Self::QueryObjectReply),
            Self::QueryServiceVersion(msg) => msg.apply_context(ctx).map(Self::QueryServiceVersion),
            Self::QueryServiceVersionReply(msg) => {
                msg.apply_context(ctx).map(Self::QueryServiceVersionReply)
            }
            Self::CreateChannel(msg) => msg.apply_context(ctx).map(Self::CreateChannel),
            Self::CreateChannelReply(msg) => msg.apply_context(ctx).map(Self::CreateChannelReply),
            Self::CloseChannelEnd(msg) => msg.apply_context(ctx).map(Self::CloseChannelEnd),
            Self::CloseChannelEndReply(msg) => {
                msg.apply_context(ctx).map(Self::CloseChannelEndReply)
            }
            Self::ChannelEndClosed(msg) => msg.apply_context(ctx).map(Self::ChannelEndClosed),
            Self::ClaimChannelEnd(msg) => msg.apply_context(ctx).map(Self::ClaimChannelEnd),
            Self::ClaimChannelEndReply(msg) => {
                msg.apply_context(ctx).map(Self::ClaimChannelEndReply)
            }
            Self::ChannelEndClaimed(msg) => msg.apply_context(ctx).map(Self::ChannelEndClaimed),
            Self::SendItem(msg) => msg.apply_context(ctx).map(Self::SendItem),
            Self::ItemReceived(msg) => msg.apply_context(ctx).map(Self::ItemReceived),
            Self::AddChannelCapacity(msg) => msg.apply_context(ctx).map(Self::AddChannelCapacity),
            Self::Sync(msg) => msg.apply_context(ctx).map(Self::Sync),
            Self::SyncReply(msg) => msg.apply_context(ctx).map(Self::SyncReply),
            Self::ServiceDestroyed(msg) => msg.apply_context(ctx).map(Self::ServiceDestroyed),
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
            ProtoMessage::EmitEvent(msg) => msg.try_into().map(Self::EmitEvent),
            ProtoMessage::QueryObject(msg) => msg.try_into().map(Self::QueryObject),
            ProtoMessage::QueryObjectReply(msg) => msg.try_into().map(Self::QueryObjectReply),
            ProtoMessage::QueryServiceVersion(msg) => msg.try_into().map(Self::QueryServiceVersion),
            ProtoMessage::QueryServiceVersionReply(msg) => {
                msg.try_into().map(Self::QueryServiceVersionReply)
            }
            ProtoMessage::CreateChannel(msg) => msg.try_into().map(Self::CreateChannel),
            ProtoMessage::CreateChannelReply(msg) => msg.try_into().map(Self::CreateChannelReply),
            ProtoMessage::CloseChannelEnd(msg) => msg.try_into().map(Self::CloseChannelEnd),
            ProtoMessage::CloseChannelEndReply(msg) => {
                msg.try_into().map(Self::CloseChannelEndReply)
            }
            ProtoMessage::ChannelEndClosed(msg) => msg.try_into().map(Self::ChannelEndClosed),
            ProtoMessage::ClaimChannelEnd(msg) => msg.try_into().map(Self::ClaimChannelEnd),
            ProtoMessage::ClaimChannelEndReply(msg) => {
                msg.try_into().map(Self::ClaimChannelEndReply)
            }
            ProtoMessage::ChannelEndClaimed(msg) => msg.try_into().map(Self::ChannelEndClaimed),
            ProtoMessage::SendItem(msg) => msg.try_into().map(Self::SendItem),
            ProtoMessage::ItemReceived(msg) => msg.try_into().map(Self::ItemReceived),
            ProtoMessage::AddChannelCapacity(msg) => msg.try_into().map(Self::AddChannelCapacity),
            ProtoMessage::Sync(msg) => msg.try_into().map(Self::Sync),
            ProtoMessage::SyncReply(msg) => msg.try_into().map(Self::SyncReply),
            ProtoMessage::ServiceDestroyed(msg) => msg.try_into().map(Self::ServiceDestroyed),
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "end", deny_unknown_fields)]
pub enum ChannelEndWithCapacity {
    Sender,
    Receiver { capacity: u32 },
}

impl From<aldrin_proto::message::ChannelEndWithCapacity> for ChannelEndWithCapacity {
    fn from(end: aldrin_proto::message::ChannelEndWithCapacity) -> Self {
        match end {
            aldrin_proto::message::ChannelEndWithCapacity::Sender => Self::Sender,

            aldrin_proto::message::ChannelEndWithCapacity::Receiver(capacity) => {
                Self::Receiver { capacity }
            }
        }
    }
}

impl From<ChannelEndWithCapacity> for aldrin_proto::message::ChannelEndWithCapacity {
    fn from(end: ChannelEndWithCapacity) -> Self {
        match end {
            ChannelEndWithCapacity::Sender => Self::Sender,
            ChannelEndWithCapacity::Receiver { capacity } => Self::Receiver(capacity),
        }
    }
}

impl fmt::Display for ChannelEndWithCapacity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Sender => f.pad("sender"),
            Self::Receiver { .. } => f.pad("receiver"),
        }
    }
}
