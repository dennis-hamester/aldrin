use clap::ValueEnum;
use serde::Deserialize;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum MessageType {
    AddBusListenerFilter,
    AddChannelCapacity,
    CallFunction,
    CallFunctionReply,
    ChannelEndClaimed,
    ChannelEndClosed,
    ClaimChannelEnd,
    ClaimChannelEndReply,
    ClearBusListenerFilters,
    CloseChannelEnd,
    CloseChannelEndReply,
    Connect,
    ConnectReply,
    CreateBusListener,
    CreateBusListenerReply,
    CreateChannel,
    CreateChannelReply,
    CreateObject,
    CreateObjectReply,
    CreateService,
    CreateServiceReply,
    DestroyBusListener,
    DestroyBusListenerReply,
    DestroyObject,
    DestroyObjectReply,
    DestroyService,
    DestroyServiceReply,
    EmitEvent,
    ItemReceived,
    QueryServiceVersion,
    QueryServiceVersionReply,
    RemoveBusListenerFilter,
    SendItem,
    ServiceDestroyed,
    Shutdown,
    StartBusListener,
    SubscribeEvent,
    SubscribeEventReply,
    Sync,
    SyncReply,
    UnsubscribeEvent,
}

impl fmt::Display for MessageType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::AddBusListenerFilter => f.pad("add-bus-listener-filter"),
            Self::AddChannelCapacity => f.pad("add-channel-capacity"),
            Self::CallFunction => f.pad("call-function"),
            Self::CallFunctionReply => f.pad("call-function-reply"),
            Self::ChannelEndClaimed => f.pad("channel-end-claimed"),
            Self::ChannelEndClosed => f.pad("channel-end-closed"),
            Self::ClaimChannelEnd => f.pad("claim-channel-end"),
            Self::ClaimChannelEndReply => f.pad("claim-channel-end-reply"),
            Self::ClearBusListenerFilters => f.pad("clear-bus-listener-filters"),
            Self::CloseChannelEnd => f.pad("close-channel-end"),
            Self::CloseChannelEndReply => f.pad("close-channel-end-reply"),
            Self::Connect => f.pad("connect"),
            Self::ConnectReply => f.pad("connect-reply"),
            Self::CreateBusListener => f.pad("create-bus-listener"),
            Self::CreateBusListenerReply => f.pad("create-bus-listener-reply"),
            Self::CreateChannel => f.pad("create-channel"),
            Self::CreateChannelReply => f.pad("create-channel-reply"),
            Self::CreateObject => f.pad("create-object"),
            Self::CreateObjectReply => f.pad("create-object-reply"),
            Self::CreateService => f.pad("create-service"),
            Self::CreateServiceReply => f.pad("create-service-reply"),
            Self::DestroyBusListener => f.pad("destroy-bus-listener"),
            Self::DestroyBusListenerReply => f.pad("destroy-bus-listener-reply"),
            Self::DestroyObject => f.pad("destroy-object"),
            Self::DestroyObjectReply => f.pad("destroy-object-reply"),
            Self::DestroyService => f.pad("destroy-service"),
            Self::DestroyServiceReply => f.pad("destroy-service-reply"),
            Self::EmitEvent => f.pad("emit-event"),
            Self::ItemReceived => f.pad("item-received"),
            Self::QueryServiceVersion => f.pad("query-service-version"),
            Self::QueryServiceVersionReply => f.pad("query-service-version-reply"),
            Self::RemoveBusListenerFilter => f.pad("remove-bus-listener-filter"),
            Self::SendItem => f.pad("send-item"),
            Self::ServiceDestroyed => f.pad("service-destroyed"),
            Self::Shutdown => f.pad("shutdown"),
            Self::StartBusListener => f.pad("start-bus-listener"),
            Self::SubscribeEvent => f.pad("subscribe-event"),
            Self::SubscribeEventReply => f.pad("subscribe-event-reply"),
            Self::Sync => f.pad("sync"),
            Self::SyncReply => f.pad("sync-reply"),
            Self::UnsubscribeEvent => f.pad("unsubscribe-event"),
        }
    }
}
