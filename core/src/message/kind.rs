use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Copy, Clone, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum MessageKind {
    Connect = 0,
    ConnectReply = 1,
    Shutdown = 2,
    CreateObject = 3,
    CreateObjectReply = 4,
    DestroyObject = 5,
    DestroyObjectReply = 6,
    CreateService = 7,
    CreateServiceReply = 8,
    DestroyService = 9,
    DestroyServiceReply = 10,
    CallFunction = 11,
    CallFunctionReply = 12,
    SubscribeEvent = 13,
    SubscribeEventReply = 14,
    UnsubscribeEvent = 15,
    EmitEvent = 16,
    QueryServiceVersion = 17,
    QueryServiceVersionReply = 18,
    CreateChannel = 19,
    CreateChannelReply = 20,
    CloseChannelEnd = 21,
    CloseChannelEndReply = 22,
    ChannelEndClosed = 23,
    ClaimChannelEnd = 24,
    ClaimChannelEndReply = 25,
    ChannelEndClaimed = 26,
    SendItem = 27,
    ItemReceived = 28,
    AddChannelCapacity = 29,
    Sync = 30,
    SyncReply = 31,
    ServiceDestroyed = 32,
    CreateBusListener = 33,
    CreateBusListenerReply = 34,
    DestroyBusListener = 35,
    DestroyBusListenerReply = 36,
    AddBusListenerFilter = 37,
    RemoveBusListenerFilter = 38,
    ClearBusListenerFilters = 39,
    StartBusListener = 40,
    StartBusListenerReply = 41,
    StopBusListener = 42,
    StopBusListenerReply = 43,
    EmitBusEvent = 44,
    BusListenerCurrentFinished = 45,
    Connect2 = 46,
    ConnectReply2 = 47,
    AbortFunctionCall = 48,
    RegisterIntrospection = 49,
    QueryIntrospection = 50,
    QueryIntrospectionReply = 51,
    CreateService2 = 52,
    QueryServiceInfo = 53,
    QueryServiceInfoReply = 54,
    SubscribeService = 55,
    SubscribeServiceReply = 56,
    UnsubscribeService = 57,
    SubscribeAllEvents = 58,
    SubscribeAllEventsReply = 59,
    UnsubscribeAllEvents = 60,
    UnsubscribeAllEventsReply = 61,
    CallFunction2 = 62,
}

impl MessageKind {
    pub fn has_value(self) -> bool {
        match self {
            Self::Connect
            | Self::ConnectReply
            | Self::CallFunction
            | Self::CallFunctionReply
            | Self::EmitEvent
            | Self::SendItem
            | Self::ItemReceived
            | Self::Connect2
            | Self::ConnectReply2
            | Self::RegisterIntrospection
            | Self::QueryIntrospectionReply
            | Self::CreateService2
            | Self::QueryServiceInfoReply
            | Self::CallFunction2 => true,

            Self::Shutdown
            | Self::CreateObject
            | Self::CreateObjectReply
            | Self::DestroyObject
            | Self::DestroyObjectReply
            | Self::CreateService
            | Self::CreateServiceReply
            | Self::DestroyService
            | Self::DestroyServiceReply
            | Self::SubscribeEvent
            | Self::SubscribeEventReply
            | Self::UnsubscribeEvent
            | Self::QueryServiceVersion
            | Self::QueryServiceVersionReply
            | Self::CreateChannel
            | Self::CreateChannelReply
            | Self::CloseChannelEnd
            | Self::CloseChannelEndReply
            | Self::ChannelEndClosed
            | Self::ClaimChannelEnd
            | Self::ClaimChannelEndReply
            | Self::ChannelEndClaimed
            | Self::AddChannelCapacity
            | Self::Sync
            | Self::SyncReply
            | Self::ServiceDestroyed
            | Self::CreateBusListener
            | Self::CreateBusListenerReply
            | Self::DestroyBusListener
            | Self::DestroyBusListenerReply
            | Self::AddBusListenerFilter
            | Self::RemoveBusListenerFilter
            | Self::ClearBusListenerFilters
            | Self::StartBusListener
            | Self::StartBusListenerReply
            | Self::StopBusListener
            | Self::StopBusListenerReply
            | Self::EmitBusEvent
            | Self::BusListenerCurrentFinished
            | Self::AbortFunctionCall
            | Self::QueryIntrospection
            | Self::QueryServiceInfo
            | Self::SubscribeService
            | Self::SubscribeServiceReply
            | Self::UnsubscribeService
            | Self::SubscribeAllEvents
            | Self::SubscribeAllEventsReply
            | Self::UnsubscribeAllEvents
            | Self::UnsubscribeAllEventsReply => false,
        }
    }
}
