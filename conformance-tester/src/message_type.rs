use clap::ValueEnum;
use serde::Deserialize;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum MessageType {
    AddChannelCapacity,
    CallFunction,
    CallFunctionReply,
    ChannelEndClaimed,
    ChannelEndClosed,
    ClaimChannelEnd,
    ClaimChannelEndReply,
    CloseChannelEnd,
    CloseChannelEndReply,
    Connect,
    ConnectReply,
    CreateChannel,
    CreateChannelReply,
    CreateObject,
    CreateObjectReply,
    CreateService,
    CreateServiceReply,
    DestroyObject,
    DestroyObjectReply,
    DestroyService,
    DestroyServiceReply,
    EmitEvent,
    ItemReceived,
    ObjectCreatedEvent,
    ObjectDestroyedEvent,
    QueryObject,
    QueryObjectReply,
    QueryServiceVersion,
    QueryServiceVersionReply,
    SendItem,
    ServiceCreatedEvent,
    ServiceDestroyedEvent,
    Shutdown,
    SubscribeEvent,
    SubscribeEventReply,
    SubscribeObjects,
    SubscribeObjectsReply,
    SubscribeServices,
    SubscribeServicesReply,
    Sync,
    SyncReply,
    UnsubscribeEvent,
    UnsubscribeObjects,
    UnsubscribeServices,
}

impl fmt::Display for MessageType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::AddChannelCapacity => f.pad("add-channel-capacity"),
            Self::CallFunction => f.pad("call-function"),
            Self::CallFunctionReply => f.pad("call-function-reply"),
            Self::ChannelEndClaimed => f.pad("channel-end-claimed"),
            Self::ChannelEndClosed => f.pad("channel-end-closed"),
            Self::ClaimChannelEnd => f.pad("claim-channel-end"),
            Self::ClaimChannelEndReply => f.pad("claim-channel-end-reply"),
            Self::CloseChannelEnd => f.pad("close-channel-end"),
            Self::CloseChannelEndReply => f.pad("close-channel-end-reply"),
            Self::Connect => f.pad("connect"),
            Self::ConnectReply => f.pad("connect-reply"),
            Self::CreateChannel => f.pad("create-channel"),
            Self::CreateChannelReply => f.pad("create-channel-reply"),
            Self::CreateObject => f.pad("create-object"),
            Self::CreateObjectReply => f.pad("create-object-reply"),
            Self::CreateService => f.pad("create-service"),
            Self::CreateServiceReply => f.pad("create-service-reply"),
            Self::DestroyObject => f.pad("destroy-object"),
            Self::DestroyObjectReply => f.pad("destroy-object-reply"),
            Self::DestroyService => f.pad("destroy-service"),
            Self::DestroyServiceReply => f.pad("destroy-service-reply"),
            Self::EmitEvent => f.pad("emit-event"),
            Self::ItemReceived => f.pad("item-received"),
            Self::ObjectCreatedEvent => f.pad("object-created-event"),
            Self::ObjectDestroyedEvent => f.pad("object-destroyed-event"),
            Self::QueryObject => f.pad("query-object"),
            Self::QueryObjectReply => f.pad("query-object-reply"),
            Self::QueryServiceVersion => f.pad("query-service-version"),
            Self::QueryServiceVersionReply => f.pad("query-service-version-reply"),
            Self::SendItem => f.pad("send-item"),
            Self::ServiceCreatedEvent => f.pad("service-created-event"),
            Self::ServiceDestroyedEvent => f.pad("service-destroyed-event"),
            Self::Shutdown => f.pad("shutdown"),
            Self::SubscribeEvent => f.pad("subscribe-event"),
            Self::SubscribeEventReply => f.pad("subscribe-event-reply"),
            Self::SubscribeObjects => f.pad("subscribe-objects"),
            Self::SubscribeObjectsReply => f.pad("subscribe-objects-reply"),
            Self::SubscribeServices => f.pad("subscribe-services"),
            Self::SubscribeServicesReply => f.pad("subscribe-services-reply"),
            Self::Sync => f.pad("sync"),
            Self::SyncReply => f.pad("sync-reply"),
            Self::UnsubscribeEvent => f.pad("unsubscribe-event"),
            Self::UnsubscribeObjects => f.pad("unsubscribe-objects"),
            Self::UnsubscribeServices => f.pad("unsubscribe-services"),
        }
    }
}
