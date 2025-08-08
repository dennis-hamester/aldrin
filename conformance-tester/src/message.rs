mod abort_function_call;
mod add_bus_listener_filter;
mod add_channel_capacity;
mod bus_listener_current_finished;
mod bus_listener_filter;
mod call_function;
mod call_function2;
mod call_function_reply;
mod channel_end_claimed;
mod channel_end_closed;
mod claim_channel_end;
mod claim_channel_end_reply;
mod clear_bus_listener_filters;
mod close_channel_end;
mod close_channel_end_reply;
mod connect;
mod connect2;
mod connect_reply;
mod connect_reply2;
mod create_bus_listener;
mod create_bus_listener_reply;
mod create_channel;
mod create_channel_reply;
mod create_object;
mod create_object_reply;
mod create_service;
mod create_service2;
mod create_service_reply;
mod destroy_bus_listener;
mod destroy_bus_listener_reply;
mod destroy_object;
mod destroy_object_reply;
mod destroy_service;
mod destroy_service_reply;
mod emit_bus_event;
mod emit_event;
mod item_received;
mod query_introspection;
mod query_introspection_reply;
mod query_service_info;
mod query_service_info_reply;
mod query_service_version;
mod query_service_version_reply;
mod register_introspection;
mod remove_bus_listener_filter;
mod send_item;
mod service_destroyed;
mod shutdown;
mod start_bus_listener;
mod start_bus_listener_reply;
mod stop_bus_listener;
mod stop_bus_listener_reply;
mod subscribe_all_events;
mod subscribe_all_events_reply;
mod subscribe_event;
mod subscribe_event_reply;
mod subscribe_service;
mod subscribe_service_reply;
mod sync;
mod sync_reply;
mod unsubscribe_all_events;
mod unsubscribe_all_events_reply;
mod unsubscribe_event;
mod unsubscribe_service;

use crate::context::Context;
use crate::uuid_ref::UuidRef;
use aldrin_core::message::Message as ProtoMessage;
use aldrin_core::{ServiceInfo as CoreServiceInfo, TypeId};
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};
use std::fmt;

pub(crate) use abort_function_call::AbortFunctionCall;
pub(crate) use add_bus_listener_filter::AddBusListenerFilter;
pub(crate) use add_channel_capacity::AddChannelCapacity;
pub(crate) use bus_listener_current_finished::BusListenerCurrentFinished;
pub(crate) use call_function::CallFunction;
pub(crate) use call_function2::CallFunction2;
pub(crate) use call_function_reply::CallFunctionReply;
pub(crate) use channel_end_claimed::ChannelEndClaimed;
pub(crate) use channel_end_closed::ChannelEndClosed;
pub(crate) use claim_channel_end::ClaimChannelEnd;
pub(crate) use claim_channel_end_reply::{ClaimChannelEndReply, ClaimChannelEndResult};
pub(crate) use clear_bus_listener_filters::ClearBusListenerFilters;
pub(crate) use close_channel_end::CloseChannelEnd;
pub(crate) use close_channel_end_reply::{CloseChannelEndReply, CloseChannelEndResult};
pub(crate) use connect::Connect;
pub(crate) use connect2::Connect2;
pub(crate) use connect_reply::ConnectReply;
pub(crate) use connect_reply2::{ConnectReply2, ConnectResult};
pub(crate) use create_bus_listener::CreateBusListener;
pub(crate) use create_bus_listener_reply::CreateBusListenerReply;
pub(crate) use create_channel::CreateChannel;
pub(crate) use create_channel_reply::CreateChannelReply;
pub(crate) use create_object::CreateObject;
pub(crate) use create_object_reply::{CreateObjectReply, CreateObjectResult};
pub(crate) use create_service::CreateService;
pub(crate) use create_service2::CreateService2;
pub(crate) use create_service_reply::{CreateServiceReply, CreateServiceResult};
pub(crate) use destroy_bus_listener::DestroyBusListener;
pub(crate) use destroy_bus_listener_reply::{DestroyBusListenerReply, DestroyBusListenerResult};
pub(crate) use destroy_object::DestroyObject;
pub(crate) use destroy_object_reply::{DestroyObjectReply, DestroyObjectResult};
pub(crate) use destroy_service::DestroyService;
pub(crate) use destroy_service_reply::{DestroyServiceReply, DestroyServiceResult};
pub(crate) use emit_bus_event::EmitBusEvent;
pub(crate) use emit_event::EmitEvent;
pub(crate) use item_received::ItemReceived;
pub(crate) use query_introspection::QueryIntrospection;
pub(crate) use query_introspection_reply::QueryIntrospectionReply;
pub(crate) use query_service_info::QueryServiceInfo;
pub(crate) use query_service_info_reply::QueryServiceInfoReply;
pub(crate) use query_service_version::QueryServiceVersion;
pub(crate) use query_service_version_reply::QueryServiceVersionReply;
pub(crate) use register_introspection::RegisterIntrospection;
pub(crate) use remove_bus_listener_filter::RemoveBusListenerFilter;
pub(crate) use send_item::SendItem;
pub(crate) use service_destroyed::ServiceDestroyed;
pub(crate) use shutdown::Shutdown;
pub(crate) use start_bus_listener::StartBusListener;
pub(crate) use start_bus_listener_reply::{StartBusListenerReply, StartBusListenerResult};
pub(crate) use stop_bus_listener::StopBusListener;
pub(crate) use stop_bus_listener_reply::{StopBusListenerReply, StopBusListenerResult};
pub(crate) use subscribe_all_events::SubscribeAllEvents;
pub(crate) use subscribe_all_events_reply::{SubscribeAllEventsReply, SubscribeAllEventsResult};
pub(crate) use subscribe_event::SubscribeEvent;
pub(crate) use subscribe_event_reply::{SubscribeEventReply, SubscribeEventResult};
pub(crate) use subscribe_service::SubscribeService;
pub(crate) use subscribe_service_reply::SubscribeServiceReply;
pub(crate) use sync::Sync;
pub(crate) use sync_reply::SyncReply;
pub(crate) use unsubscribe_all_events::UnsubscribeAllEvents;
pub(crate) use unsubscribe_all_events_reply::UnsubscribeAllEventsReply;
pub(crate) use unsubscribe_event::UnsubscribeEvent;
pub(crate) use unsubscribe_service::UnsubscribeService;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "message")]
pub(crate) enum Message {
    Connect(Connect),
    ConnectReply(ConnectReply),
    Shutdown(Shutdown),
    CreateObject(CreateObject),
    CreateObjectReply(CreateObjectReply),
    DestroyObject(DestroyObject),
    DestroyObjectReply(DestroyObjectReply),
    CreateService(CreateService),
    CreateServiceReply(CreateServiceReply),
    DestroyService(DestroyService),
    DestroyServiceReply(DestroyServiceReply),
    CallFunction(CallFunction),
    CallFunctionReply(CallFunctionReply),
    SubscribeEvent(SubscribeEvent),
    SubscribeEventReply(SubscribeEventReply),
    UnsubscribeEvent(UnsubscribeEvent),
    EmitEvent(EmitEvent),
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
    CreateBusListener(CreateBusListener),
    CreateBusListenerReply(CreateBusListenerReply),
    DestroyBusListener(DestroyBusListener),
    DestroyBusListenerReply(DestroyBusListenerReply),
    AddBusListenerFilter(AddBusListenerFilter),
    RemoveBusListenerFilter(RemoveBusListenerFilter),
    ClearBusListenerFilters(ClearBusListenerFilters),
    StartBusListener(StartBusListener),
    StartBusListenerReply(StartBusListenerReply),
    StopBusListener(StopBusListener),
    StopBusListenerReply(StopBusListenerReply),
    EmitBusEvent(EmitBusEvent),
    BusListenerCurrentFinished(BusListenerCurrentFinished),
    Connect2(Connect2),
    ConnectReply2(ConnectReply2),
    AbortFunctionCall(AbortFunctionCall),
    RegisterIntrospection(RegisterIntrospection),
    QueryIntrospection(QueryIntrospection),
    QueryIntrospectionReply(QueryIntrospectionReply),
    CreateService2(CreateService2),
    QueryServiceInfo(QueryServiceInfo),
    QueryServiceInfoReply(QueryServiceInfoReply),
    SubscribeService(SubscribeService),
    SubscribeServiceReply(SubscribeServiceReply),
    UnsubscribeService(UnsubscribeService),
    SubscribeAllEvents(SubscribeAllEvents),
    SubscribeAllEventsReply(SubscribeAllEventsReply),
    UnsubscribeAllEvents(UnsubscribeAllEvents),
    UnsubscribeAllEventsReply(UnsubscribeAllEventsReply),
    CallFunction2(CallFunction2),
}

impl Message {
    pub(crate) fn to_core(&self, ctx: &Context) -> Result<ProtoMessage> {
        match self {
            Self::Connect(msg) => msg.to_core(ctx).map(ProtoMessage::Connect),
            Self::ConnectReply(msg) => msg.to_core(ctx).map(ProtoMessage::ConnectReply),
            Self::Shutdown(msg) => msg.to_core(ctx).map(ProtoMessage::Shutdown),
            Self::CreateObject(msg) => msg.to_core(ctx).map(ProtoMessage::CreateObject),
            Self::CreateObjectReply(msg) => msg.to_core(ctx).map(ProtoMessage::CreateObjectReply),
            Self::DestroyObject(msg) => msg.to_core(ctx).map(ProtoMessage::DestroyObject),
            Self::DestroyObjectReply(msg) => msg.to_core(ctx).map(ProtoMessage::DestroyObjectReply),
            Self::CreateService(msg) => msg.to_core(ctx).map(ProtoMessage::CreateService),
            Self::CreateServiceReply(msg) => msg.to_core(ctx).map(ProtoMessage::CreateServiceReply),
            Self::DestroyService(msg) => msg.to_core(ctx).map(ProtoMessage::DestroyService),
            Self::DestroyServiceReply(msg) => {
                msg.to_core(ctx).map(ProtoMessage::DestroyServiceReply)
            }
            Self::CallFunction(msg) => msg.to_core(ctx).map(ProtoMessage::CallFunction),
            Self::CallFunctionReply(msg) => msg.to_core(ctx).map(ProtoMessage::CallFunctionReply),
            Self::SubscribeEvent(msg) => msg.to_core(ctx).map(ProtoMessage::SubscribeEvent),
            Self::SubscribeEventReply(msg) => {
                msg.to_core(ctx).map(ProtoMessage::SubscribeEventReply)
            }
            Self::UnsubscribeEvent(msg) => msg.to_core(ctx).map(ProtoMessage::UnsubscribeEvent),
            Self::EmitEvent(msg) => msg.to_core(ctx).map(ProtoMessage::EmitEvent),
            Self::QueryServiceVersion(msg) => {
                msg.to_core(ctx).map(ProtoMessage::QueryServiceVersion)
            }
            Self::QueryServiceVersionReply(msg) => {
                msg.to_core(ctx).map(ProtoMessage::QueryServiceVersionReply)
            }
            Self::CreateChannel(msg) => msg.to_core(ctx).map(ProtoMessage::CreateChannel),
            Self::CreateChannelReply(msg) => msg.to_core(ctx).map(ProtoMessage::CreateChannelReply),
            Self::CloseChannelEnd(msg) => msg.to_core(ctx).map(ProtoMessage::CloseChannelEnd),
            Self::CloseChannelEndReply(msg) => {
                msg.to_core(ctx).map(ProtoMessage::CloseChannelEndReply)
            }
            Self::ChannelEndClosed(msg) => msg.to_core(ctx).map(ProtoMessage::ChannelEndClosed),
            Self::ClaimChannelEnd(msg) => msg.to_core(ctx).map(ProtoMessage::ClaimChannelEnd),
            Self::ClaimChannelEndReply(msg) => {
                msg.to_core(ctx).map(ProtoMessage::ClaimChannelEndReply)
            }
            Self::ChannelEndClaimed(msg) => msg.to_core(ctx).map(ProtoMessage::ChannelEndClaimed),
            Self::SendItem(msg) => msg.to_core(ctx).map(ProtoMessage::SendItem),
            Self::ItemReceived(msg) => msg.to_core(ctx).map(ProtoMessage::ItemReceived),
            Self::AddChannelCapacity(msg) => msg.to_core(ctx).map(ProtoMessage::AddChannelCapacity),
            Self::Sync(msg) => msg.to_core(ctx).map(ProtoMessage::Sync),
            Self::SyncReply(msg) => msg.to_core(ctx).map(ProtoMessage::SyncReply),
            Self::ServiceDestroyed(msg) => msg.to_core(ctx).map(ProtoMessage::ServiceDestroyed),
            Self::CreateBusListener(msg) => msg.to_core(ctx).map(ProtoMessage::CreateBusListener),
            Self::CreateBusListenerReply(msg) => {
                msg.to_core(ctx).map(ProtoMessage::CreateBusListenerReply)
            }
            Self::DestroyBusListener(msg) => msg.to_core(ctx).map(ProtoMessage::DestroyBusListener),
            Self::DestroyBusListenerReply(msg) => {
                msg.to_core(ctx).map(ProtoMessage::DestroyBusListenerReply)
            }
            Self::AddBusListenerFilter(msg) => {
                msg.to_core(ctx).map(ProtoMessage::AddBusListenerFilter)
            }
            Self::RemoveBusListenerFilter(msg) => {
                msg.to_core(ctx).map(ProtoMessage::RemoveBusListenerFilter)
            }
            Self::ClearBusListenerFilters(msg) => {
                msg.to_core(ctx).map(ProtoMessage::ClearBusListenerFilters)
            }
            Self::StartBusListener(msg) => msg.to_core(ctx).map(ProtoMessage::StartBusListener),
            Self::StartBusListenerReply(msg) => {
                msg.to_core(ctx).map(ProtoMessage::StartBusListenerReply)
            }
            Self::StopBusListener(msg) => msg.to_core(ctx).map(ProtoMessage::StopBusListener),
            Self::StopBusListenerReply(msg) => {
                msg.to_core(ctx).map(ProtoMessage::StopBusListenerReply)
            }
            Self::EmitBusEvent(msg) => msg.to_core(ctx).map(ProtoMessage::EmitBusEvent),
            Self::BusListenerCurrentFinished(msg) => msg
                .to_core(ctx)
                .map(ProtoMessage::BusListenerCurrentFinished),
            Self::Connect2(msg) => msg.to_core(ctx).map(ProtoMessage::Connect2),
            Self::ConnectReply2(msg) => msg.to_core(ctx).map(ProtoMessage::ConnectReply2),
            Self::AbortFunctionCall(msg) => msg.to_core(ctx).map(ProtoMessage::AbortFunctionCall),
            Self::RegisterIntrospection(msg) => {
                msg.to_core(ctx).map(ProtoMessage::RegisterIntrospection)
            }
            Self::QueryIntrospection(msg) => msg.to_core(ctx).map(ProtoMessage::QueryIntrospection),
            Self::QueryIntrospectionReply(msg) => {
                msg.to_core(ctx).map(ProtoMessage::QueryIntrospectionReply)
            }
            Self::CreateService2(msg) => msg.to_core(ctx).map(ProtoMessage::CreateService2),
            Self::QueryServiceInfo(msg) => msg.to_core(ctx).map(ProtoMessage::QueryServiceInfo),
            Self::QueryServiceInfoReply(msg) => {
                msg.to_core(ctx).map(ProtoMessage::QueryServiceInfoReply)
            }
            Self::SubscribeService(msg) => msg.to_core(ctx).map(ProtoMessage::SubscribeService),
            Self::SubscribeServiceReply(msg) => {
                msg.to_core(ctx).map(ProtoMessage::SubscribeServiceReply)
            }
            Self::UnsubscribeService(msg) => msg.to_core(ctx).map(ProtoMessage::UnsubscribeService),
            Self::SubscribeAllEvents(msg) => msg.to_core(ctx).map(ProtoMessage::SubscribeAllEvents),
            Self::SubscribeAllEventsReply(msg) => {
                msg.to_core(ctx).map(ProtoMessage::SubscribeAllEventsReply)
            }
            Self::UnsubscribeAllEvents(msg) => {
                msg.to_core(ctx).map(ProtoMessage::UnsubscribeAllEvents)
            }
            Self::UnsubscribeAllEventsReply(msg) => msg
                .to_core(ctx)
                .map(ProtoMessage::UnsubscribeAllEventsReply),
            Self::CallFunction2(msg) => msg.to_core(ctx).map(ProtoMessage::CallFunction2),
        }
    }

    pub(crate) fn matches(&self, other: &Self, ctx: &Context) -> Result<bool> {
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
            (Self::CreateService(msg), Self::CreateService(other)) => msg.matches(other, ctx),
            (Self::CreateServiceReply(msg), Self::CreateServiceReply(other)) => {
                msg.matches(other, ctx)
            }
            (Self::DestroyService(msg), Self::DestroyService(other)) => msg.matches(other, ctx),
            (Self::DestroyServiceReply(msg), Self::DestroyServiceReply(other)) => {
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
            (Self::CreateBusListener(msg), Self::CreateBusListener(other)) => {
                msg.matches(other, ctx)
            }
            (Self::CreateBusListenerReply(msg), Self::CreateBusListenerReply(other)) => {
                msg.matches(other, ctx)
            }
            (Self::DestroyBusListener(msg), Self::DestroyBusListener(other)) => {
                msg.matches(other, ctx)
            }
            (Self::DestroyBusListenerReply(msg), Self::DestroyBusListenerReply(other)) => {
                msg.matches(other, ctx)
            }
            (Self::AddBusListenerFilter(msg), Self::AddBusListenerFilter(other)) => {
                msg.matches(other, ctx)
            }
            (Self::RemoveBusListenerFilter(msg), Self::RemoveBusListenerFilter(other)) => {
                msg.matches(other, ctx)
            }
            (Self::ClearBusListenerFilters(msg), Self::ClearBusListenerFilters(other)) => {
                msg.matches(other, ctx)
            }
            (Self::StartBusListener(msg), Self::StartBusListener(other)) => msg.matches(other, ctx),
            (Self::StartBusListenerReply(msg), Self::StartBusListenerReply(other)) => {
                msg.matches(other, ctx)
            }
            (Self::StopBusListener(msg), Self::StopBusListener(other)) => msg.matches(other, ctx),
            (Self::StopBusListenerReply(msg), Self::StopBusListenerReply(other)) => {
                msg.matches(other, ctx)
            }
            (Self::EmitBusEvent(msg), Self::EmitBusEvent(other)) => msg.matches(other, ctx),
            (Self::BusListenerCurrentFinished(msg), Self::BusListenerCurrentFinished(other)) => {
                msg.matches(other, ctx)
            }
            (Self::Connect2(msg), Self::Connect2(other)) => msg.matches(other, ctx),
            (Self::ConnectReply2(msg), Self::ConnectReply2(other)) => msg.matches(other, ctx),
            (Self::AbortFunctionCall(msg), Self::AbortFunctionCall(other)) => {
                msg.matches(other, ctx)
            }
            (Self::RegisterIntrospection(msg), Self::RegisterIntrospection(other)) => {
                msg.matches(other, ctx)
            }
            (Self::QueryIntrospection(msg), Self::QueryIntrospection(other)) => {
                msg.matches(other, ctx)
            }
            (Self::QueryIntrospectionReply(msg), Self::QueryIntrospectionReply(other)) => {
                msg.matches(other, ctx)
            }
            (Self::CreateService2(msg), Self::CreateService2(other)) => msg.matches(other, ctx),
            (Self::QueryServiceInfo(msg), Self::QueryServiceInfo(other)) => msg.matches(other, ctx),
            (Self::QueryServiceInfoReply(msg), Self::QueryServiceInfoReply(other)) => {
                msg.matches(other, ctx)
            }
            (Self::SubscribeService(msg), Self::SubscribeService(other)) => msg.matches(other, ctx),
            (Self::SubscribeServiceReply(msg), Self::SubscribeServiceReply(other)) => {
                msg.matches(other, ctx)
            }
            (Self::UnsubscribeService(msg), Self::UnsubscribeService(other)) => {
                msg.matches(other, ctx)
            }
            (Self::SubscribeAllEvents(msg), Self::SubscribeAllEvents(other)) => {
                msg.matches(other, ctx)
            }
            (Self::SubscribeAllEventsReply(msg), Self::SubscribeAllEventsReply(other)) => {
                msg.matches(other, ctx)
            }
            (Self::UnsubscribeAllEvents(msg), Self::UnsubscribeAllEvents(other)) => {
                msg.matches(other, ctx)
            }
            (Self::UnsubscribeAllEventsReply(msg), Self::UnsubscribeAllEventsReply(other)) => {
                msg.matches(other, ctx)
            }
            (Self::CallFunction2(msg), Self::CallFunction2(other)) => msg.matches(other, ctx),
            _ => Ok(false),
        }
    }

    pub(crate) fn update_context(&self, other: &Self, ctx: &mut Context) -> Result<()> {
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
            (Self::CreateBusListener(msg), Self::CreateBusListener(other)) => {
                msg.update_context(other, ctx)
            }
            (Self::CreateBusListenerReply(msg), Self::CreateBusListenerReply(other)) => {
                msg.update_context(other, ctx)
            }
            (Self::DestroyBusListener(msg), Self::DestroyBusListener(other)) => {
                msg.update_context(other, ctx)
            }
            (Self::DestroyBusListenerReply(msg), Self::DestroyBusListenerReply(other)) => {
                msg.update_context(other, ctx)
            }
            (Self::AddBusListenerFilter(msg), Self::AddBusListenerFilter(other)) => {
                msg.update_context(other, ctx)
            }
            (Self::RemoveBusListenerFilter(msg), Self::RemoveBusListenerFilter(other)) => {
                msg.update_context(other, ctx)
            }
            (Self::ClearBusListenerFilters(msg), Self::ClearBusListenerFilters(other)) => {
                msg.update_context(other, ctx)
            }
            (Self::StartBusListener(msg), Self::StartBusListener(other)) => {
                msg.update_context(other, ctx)
            }
            (Self::StartBusListenerReply(msg), Self::StartBusListenerReply(other)) => {
                msg.update_context(other, ctx)
            }
            (Self::StopBusListener(msg), Self::StopBusListener(other)) => {
                msg.update_context(other, ctx)
            }
            (Self::StopBusListenerReply(msg), Self::StopBusListenerReply(other)) => {
                msg.update_context(other, ctx)
            }
            (Self::EmitBusEvent(msg), Self::EmitBusEvent(other)) => msg.update_context(other, ctx),
            (Self::BusListenerCurrentFinished(msg), Self::BusListenerCurrentFinished(other)) => {
                msg.update_context(other, ctx)
            }
            (Self::Connect2(msg), Self::Connect2(other)) => msg.update_context(other, ctx),
            (Self::ConnectReply2(msg), Self::ConnectReply2(other)) => {
                msg.update_context(other, ctx)
            }
            (Self::AbortFunctionCall(msg), Self::AbortFunctionCall(other)) => {
                msg.update_context(other, ctx)
            }
            (Self::RegisterIntrospection(msg), Self::RegisterIntrospection(other)) => {
                msg.update_context(other, ctx)
            }
            (Self::QueryIntrospection(msg), Self::QueryIntrospection(other)) => {
                msg.update_context(other, ctx)
            }
            (Self::QueryIntrospectionReply(msg), Self::QueryIntrospectionReply(other)) => {
                msg.update_context(other, ctx)
            }
            (Self::CreateService2(msg), Self::CreateService2(other)) => {
                msg.update_context(other, ctx)
            }
            (Self::QueryServiceInfo(msg), Self::QueryServiceInfo(other)) => {
                msg.update_context(other, ctx)
            }
            (Self::QueryServiceInfoReply(msg), Self::QueryServiceInfoReply(other)) => {
                msg.update_context(other, ctx)
            }
            (Self::SubscribeService(msg), Self::SubscribeService(other)) => {
                msg.update_context(other, ctx)
            }
            (Self::SubscribeServiceReply(msg), Self::SubscribeServiceReply(other)) => {
                msg.update_context(other, ctx)
            }
            (Self::UnsubscribeService(msg), Self::UnsubscribeService(other)) => {
                msg.update_context(other, ctx)
            }
            (Self::SubscribeAllEvents(msg), Self::SubscribeAllEvents(other)) => {
                msg.update_context(other, ctx)
            }
            (Self::SubscribeAllEventsReply(msg), Self::SubscribeAllEventsReply(other)) => {
                msg.update_context(other, ctx)
            }
            (Self::UnsubscribeAllEvents(msg), Self::UnsubscribeAllEvents(other)) => {
                msg.update_context(other, ctx)
            }
            (Self::UnsubscribeAllEventsReply(msg), Self::UnsubscribeAllEventsReply(other)) => {
                msg.update_context(other, ctx)
            }
            (Self::CallFunction2(msg), Self::CallFunction2(other)) => {
                msg.update_context(other, ctx)
            }
            _ => unreachable!(),
        }
    }

    pub(crate) fn apply_context(&self, ctx: &Context) -> Result<Self> {
        match self {
            Self::Connect(msg) => msg.apply_context(ctx).map(Self::Connect),
            Self::ConnectReply(msg) => msg.apply_context(ctx).map(Self::ConnectReply),
            Self::Shutdown(msg) => msg.apply_context(ctx).map(Self::Shutdown),
            Self::CreateObject(msg) => msg.apply_context(ctx).map(Self::CreateObject),
            Self::CreateObjectReply(msg) => msg.apply_context(ctx).map(Self::CreateObjectReply),
            Self::DestroyObject(msg) => msg.apply_context(ctx).map(Self::DestroyObject),
            Self::DestroyObjectReply(msg) => msg.apply_context(ctx).map(Self::DestroyObjectReply),
            Self::CreateService(msg) => msg.apply_context(ctx).map(Self::CreateService),
            Self::CreateServiceReply(msg) => msg.apply_context(ctx).map(Self::CreateServiceReply),
            Self::DestroyService(msg) => msg.apply_context(ctx).map(Self::DestroyService),
            Self::DestroyServiceReply(msg) => msg.apply_context(ctx).map(Self::DestroyServiceReply),
            Self::CallFunction(msg) => msg.apply_context(ctx).map(Self::CallFunction),
            Self::CallFunctionReply(msg) => msg.apply_context(ctx).map(Self::CallFunctionReply),
            Self::SubscribeEvent(msg) => msg.apply_context(ctx).map(Self::SubscribeEvent),
            Self::SubscribeEventReply(msg) => msg.apply_context(ctx).map(Self::SubscribeEventReply),
            Self::UnsubscribeEvent(msg) => msg.apply_context(ctx).map(Self::UnsubscribeEvent),
            Self::EmitEvent(msg) => msg.apply_context(ctx).map(Self::EmitEvent),
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
            Self::CreateBusListener(msg) => msg.apply_context(ctx).map(Self::CreateBusListener),
            Self::CreateBusListenerReply(msg) => {
                msg.apply_context(ctx).map(Self::CreateBusListenerReply)
            }
            Self::DestroyBusListener(msg) => msg.apply_context(ctx).map(Self::DestroyBusListener),
            Self::DestroyBusListenerReply(msg) => {
                msg.apply_context(ctx).map(Self::DestroyBusListenerReply)
            }
            Self::AddBusListenerFilter(msg) => {
                msg.apply_context(ctx).map(Self::AddBusListenerFilter)
            }
            Self::RemoveBusListenerFilter(msg) => {
                msg.apply_context(ctx).map(Self::RemoveBusListenerFilter)
            }
            Self::ClearBusListenerFilters(msg) => {
                msg.apply_context(ctx).map(Self::ClearBusListenerFilters)
            }
            Self::StartBusListener(msg) => msg.apply_context(ctx).map(Self::StartBusListener),
            Self::StartBusListenerReply(msg) => {
                msg.apply_context(ctx).map(Self::StartBusListenerReply)
            }
            Self::StopBusListener(msg) => msg.apply_context(ctx).map(Self::StopBusListener),
            Self::StopBusListenerReply(msg) => {
                msg.apply_context(ctx).map(Self::StopBusListenerReply)
            }
            Self::EmitBusEvent(msg) => msg.apply_context(ctx).map(Self::EmitBusEvent),
            Self::BusListenerCurrentFinished(msg) => {
                msg.apply_context(ctx).map(Self::BusListenerCurrentFinished)
            }
            Self::Connect2(msg) => msg.apply_context(ctx).map(Self::Connect2),
            Self::ConnectReply2(msg) => msg.apply_context(ctx).map(Self::ConnectReply2),
            Self::AbortFunctionCall(msg) => msg.apply_context(ctx).map(Self::AbortFunctionCall),
            Self::RegisterIntrospection(msg) => {
                msg.apply_context(ctx).map(Self::RegisterIntrospection)
            }
            Self::QueryIntrospection(msg) => msg.apply_context(ctx).map(Self::QueryIntrospection),
            Self::QueryIntrospectionReply(msg) => {
                msg.apply_context(ctx).map(Self::QueryIntrospectionReply)
            }
            Self::CreateService2(msg) => msg.apply_context(ctx).map(Self::CreateService2),
            Self::QueryServiceInfo(msg) => msg.apply_context(ctx).map(Self::QueryServiceInfo),
            Self::QueryServiceInfoReply(msg) => {
                msg.apply_context(ctx).map(Self::QueryServiceInfoReply)
            }
            Self::SubscribeService(msg) => msg.apply_context(ctx).map(Self::SubscribeService),
            Self::SubscribeServiceReply(msg) => {
                msg.apply_context(ctx).map(Self::SubscribeServiceReply)
            }
            Self::UnsubscribeService(msg) => msg.apply_context(ctx).map(Self::UnsubscribeService),
            Self::SubscribeAllEvents(msg) => msg.apply_context(ctx).map(Self::SubscribeAllEvents),
            Self::SubscribeAllEventsReply(msg) => {
                msg.apply_context(ctx).map(Self::SubscribeAllEventsReply)
            }
            Self::UnsubscribeAllEvents(msg) => {
                msg.apply_context(ctx).map(Self::UnsubscribeAllEvents)
            }
            Self::UnsubscribeAllEventsReply(msg) => {
                msg.apply_context(ctx).map(Self::UnsubscribeAllEventsReply)
            }
            Self::CallFunction2(msg) => msg.apply_context(ctx).map(Self::CallFunction2),
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
            ProtoMessage::CreateService(msg) => msg.try_into().map(Self::CreateService),
            ProtoMessage::CreateServiceReply(msg) => msg.try_into().map(Self::CreateServiceReply),
            ProtoMessage::DestroyService(msg) => msg.try_into().map(Self::DestroyService),
            ProtoMessage::DestroyServiceReply(msg) => msg.try_into().map(Self::DestroyServiceReply),
            ProtoMessage::CallFunction(msg) => msg.try_into().map(Self::CallFunction),
            ProtoMessage::CallFunctionReply(msg) => msg.try_into().map(Self::CallFunctionReply),
            ProtoMessage::SubscribeEvent(msg) => msg.try_into().map(Self::SubscribeEvent),
            ProtoMessage::SubscribeEventReply(msg) => msg.try_into().map(Self::SubscribeEventReply),
            ProtoMessage::UnsubscribeEvent(msg) => msg.try_into().map(Self::UnsubscribeEvent),
            ProtoMessage::EmitEvent(msg) => msg.try_into().map(Self::EmitEvent),
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
            ProtoMessage::CreateBusListener(msg) => msg.try_into().map(Self::CreateBusListener),
            ProtoMessage::CreateBusListenerReply(msg) => {
                msg.try_into().map(Self::CreateBusListenerReply)
            }
            ProtoMessage::DestroyBusListener(msg) => msg.try_into().map(Self::DestroyBusListener),
            ProtoMessage::DestroyBusListenerReply(msg) => {
                msg.try_into().map(Self::DestroyBusListenerReply)
            }
            ProtoMessage::AddBusListenerFilter(msg) => {
                msg.try_into().map(Self::AddBusListenerFilter)
            }
            ProtoMessage::RemoveBusListenerFilter(msg) => {
                msg.try_into().map(Self::RemoveBusListenerFilter)
            }
            ProtoMessage::ClearBusListenerFilters(msg) => {
                msg.try_into().map(Self::ClearBusListenerFilters)
            }
            ProtoMessage::StartBusListener(msg) => msg.try_into().map(Self::StartBusListener),
            ProtoMessage::StartBusListenerReply(msg) => {
                msg.try_into().map(Self::StartBusListenerReply)
            }
            ProtoMessage::StopBusListener(msg) => msg.try_into().map(Self::StopBusListener),
            ProtoMessage::StopBusListenerReply(msg) => {
                msg.try_into().map(Self::StopBusListenerReply)
            }
            ProtoMessage::EmitBusEvent(msg) => msg.try_into().map(Self::EmitBusEvent),
            ProtoMessage::BusListenerCurrentFinished(msg) => {
                msg.try_into().map(Self::BusListenerCurrentFinished)
            }
            ProtoMessage::Connect2(msg) => msg.try_into().map(Self::Connect2),
            ProtoMessage::ConnectReply2(msg) => msg.try_into().map(Self::ConnectReply2),
            ProtoMessage::AbortFunctionCall(msg) => msg.try_into().map(Self::AbortFunctionCall),
            ProtoMessage::RegisterIntrospection(msg) => {
                msg.try_into().map(Self::RegisterIntrospection)
            }
            ProtoMessage::QueryIntrospection(msg) => msg.try_into().map(Self::QueryIntrospection),
            ProtoMessage::QueryIntrospectionReply(msg) => {
                msg.try_into().map(Self::QueryIntrospectionReply)
            }
            ProtoMessage::CreateService2(msg) => msg.try_into().map(Self::CreateService2),
            ProtoMessage::QueryServiceInfo(msg) => msg.try_into().map(Self::QueryServiceInfo),
            ProtoMessage::QueryServiceInfoReply(msg) => {
                msg.try_into().map(Self::QueryServiceInfoReply)
            }
            ProtoMessage::SubscribeService(msg) => msg.try_into().map(Self::SubscribeService),
            ProtoMessage::SubscribeServiceReply(msg) => {
                msg.try_into().map(Self::SubscribeServiceReply)
            }
            ProtoMessage::UnsubscribeService(msg) => msg.try_into().map(Self::UnsubscribeService),
            ProtoMessage::SubscribeAllEvents(msg) => msg.try_into().map(Self::SubscribeAllEvents),
            ProtoMessage::SubscribeAllEventsReply(msg) => {
                msg.try_into().map(Self::SubscribeAllEventsReply)
            }
            ProtoMessage::UnsubscribeAllEvents(msg) => {
                msg.try_into().map(Self::UnsubscribeAllEvents)
            }
            ProtoMessage::UnsubscribeAllEventsReply(msg) => {
                msg.try_into().map(Self::UnsubscribeAllEventsReply)
            }
            ProtoMessage::CallFunction2(msg) => msg.try_into().map(Self::CallFunction2),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ChannelEnd {
    Sender,
    Receiver,
}

impl From<aldrin_core::ChannelEnd> for ChannelEnd {
    fn from(end: aldrin_core::ChannelEnd) -> Self {
        match end {
            aldrin_core::ChannelEnd::Sender => Self::Sender,
            aldrin_core::ChannelEnd::Receiver => Self::Receiver,
        }
    }
}

impl From<ChannelEnd> for aldrin_core::ChannelEnd {
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

impl From<aldrin_core::ChannelEndWithCapacity> for ChannelEndWithCapacity {
    fn from(end: aldrin_core::ChannelEndWithCapacity) -> Self {
        match end {
            aldrin_core::ChannelEndWithCapacity::Sender => Self::Sender,

            aldrin_core::ChannelEndWithCapacity::Receiver(capacity) => Self::Receiver { capacity },
        }
    }
}

impl From<ChannelEndWithCapacity> for aldrin_core::ChannelEndWithCapacity {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct ServiceInfo {
    pub version: u32,
    pub type_id: Option<UuidRef>,
    pub subscribe_all: Option<bool>,
}

impl ServiceInfo {
    pub(crate) fn to_core(&self, ctx: &Context) -> Result<CoreServiceInfo> {
        let mut info = CoreServiceInfo::new(self.version);

        if let Some(ref type_id) = self.type_id {
            let type_id = type_id.get(ctx).map(TypeId)?;
            info = info.set_type_id(type_id);
        }

        if let Some(subscribe_all) = self.subscribe_all {
            info = info.set_subscribe_all(subscribe_all);
        }

        Ok(info)
    }

    fn matches(&self, other: &Self, ctx: &Context) -> Result<bool> {
        let res = match (self.type_id.as_ref(), other.type_id.as_ref()) {
            (Some(type_id), Some(other)) => type_id.matches(other, ctx)?,
            (None, None) => true,
            _ => false,
        };

        Ok(res && (self.version == other.version) && (self.subscribe_all == other.subscribe_all))
    }

    pub(crate) fn update_context(&self, other: &Self, ctx: &mut Context) -> Result<()> {
        if let (Some(type_id), Some(other)) = (self.type_id.as_ref(), other.type_id.as_ref()) {
            type_id.update_context(other, ctx)
        } else {
            Ok(())
        }
    }

    pub(crate) fn apply_context(&self, ctx: &Context) -> Result<Self> {
        let type_id = self
            .type_id
            .as_ref()
            .map(|id| id.apply_context(ctx))
            .transpose()?;

        Ok(Self {
            version: self.version,
            type_id,
            subscribe_all: self.subscribe_all,
        })
    }
}

impl From<CoreServiceInfo> for ServiceInfo {
    fn from(info: CoreServiceInfo) -> Self {
        Self {
            version: info.version(),
            type_id: info.type_id().map(Into::into),
            subscribe_all: info.subscribe_all(),
        }
    }
}
