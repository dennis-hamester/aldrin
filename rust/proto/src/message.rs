use crate::ids::{
    ChannelCookie, ObjectCookie, ObjectId, ObjectUuid, ServiceCookie, ServiceId, ServiceUuid,
};
use crate::value::Value;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub enum Message {
    Connect(Connect),
    ConnectReply(ConnectReply),
    Shutdown(()),
    CreateObject(CreateObject),
    CreateObjectReply(CreateObjectReply),
    DestroyObject(DestroyObject),
    DestroyObjectReply(DestroyObjectReply),
    SubscribeObjects(SubscribeObjects),
    SubscribeObjectsReply(SubscribeObjectsReply),
    UnsubscribeObjects(()),
    ObjectCreatedEvent(ObjectCreatedEvent),
    ObjectDestroyedEvent(ObjectDestroyedEvent),
    CreateService(CreateService),
    CreateServiceReply(CreateServiceReply),
    DestroyService(DestroyService),
    DestroyServiceReply(DestroyServiceReply),
    SubscribeServices(SubscribeServices),
    SubscribeServicesReply(SubscribeServicesReply),
    UnsubscribeServices(()),
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
    DestroyChannelEnd(DestroyChannelEnd),
    DestroyChannelEndReply(DestroyChannelEndReply),
    ChannelEndDestroyed(ChannelEndDestroyed),
    ClaimChannelEnd(ClaimChannelEnd),
    ClaimChannelEndReply(ClaimChannelEndReply),
    ChannelEndClaimed(ChannelEndClaimed),
    SendItem(SendItem),
    ItemReceived(ItemReceived),
}

/// Sending or receiving end of a channel.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub enum ChannelEnd {
    /// Sending end of a channel.
    Sender,

    /// Receiving end of a channel.
    Receiver,
}

impl ChannelEnd {
    /// Returns the other end of the channel.
    ///
    /// This function maps [`Sender`](Self::Sender) to [`Receiver`](Self::Receiver) and vice versa.
    ///
    /// # Examples
    ///
    /// ```
    /// # use aldrin_proto::ChannelEnd;
    /// assert_eq!(ChannelEnd::Sender.other(), ChannelEnd::Receiver);
    /// assert_eq!(ChannelEnd::Receiver.other(), ChannelEnd::Sender);
    /// ```
    pub fn other(self) -> Self {
        match self {
            Self::Sender => Self::Receiver,
            Self::Receiver => Self::Sender,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub struct Connect {
    pub version: u32,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub enum ConnectReply {
    Ok,
    VersionMismatch(u32),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub struct CreateObject {
    pub serial: u32,
    pub uuid: ObjectUuid,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub enum CreateObjectResult {
    Ok(ObjectCookie),
    DuplicateObject,
}

impl CreateObjectResult {
    pub fn is_ok(&self) -> bool {
        match self {
            CreateObjectResult::Ok(_) => true,
            CreateObjectResult::DuplicateObject => false,
        }
    }

    pub fn is_err(&self) -> bool {
        !self.is_ok()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub struct CreateObjectReply {
    pub serial: u32,
    pub result: CreateObjectResult,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub struct DestroyObject {
    pub serial: u32,
    pub cookie: ObjectCookie,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub enum DestroyObjectResult {
    Ok,
    InvalidObject,
    ForeignObject,
}

impl DestroyObjectResult {
    pub fn is_ok(&self) -> bool {
        match self {
            DestroyObjectResult::Ok => true,
            DestroyObjectResult::InvalidObject | DestroyObjectResult::ForeignObject => false,
        }
    }

    pub fn is_err(&self) -> bool {
        !self.is_ok()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub struct DestroyObjectReply {
    pub serial: u32,
    pub result: DestroyObjectResult,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub struct SubscribeObjects {
    pub serial: Option<u32>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub struct SubscribeObjectsReply {
    pub serial: u32,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub struct ObjectCreatedEvent {
    pub id: ObjectId,
    pub serial: Option<u32>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub struct ObjectDestroyedEvent {
    pub id: ObjectId,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub struct CreateService {
    pub serial: u32,
    pub object_cookie: ObjectCookie,
    pub uuid: ServiceUuid,
    pub version: u32,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub enum CreateServiceResult {
    Ok(ServiceCookie),
    DuplicateService,
    InvalidObject,
    ForeignObject,
}

impl CreateServiceResult {
    pub fn is_ok(&self) -> bool {
        match self {
            CreateServiceResult::Ok(_) => true,
            CreateServiceResult::DuplicateService
            | CreateServiceResult::InvalidObject
            | CreateServiceResult::ForeignObject => false,
        }
    }

    pub fn is_err(&self) -> bool {
        !self.is_ok()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub struct CreateServiceReply {
    pub serial: u32,
    pub result: CreateServiceResult,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub struct DestroyService {
    pub serial: u32,
    pub cookie: ServiceCookie,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub enum DestroyServiceResult {
    Ok,
    InvalidService,
    ForeignObject,
}

impl DestroyServiceResult {
    pub fn is_ok(&self) -> bool {
        match self {
            DestroyServiceResult::Ok => true,
            DestroyServiceResult::InvalidService | DestroyServiceResult::ForeignObject => false,
        }
    }

    pub fn is_err(&self) -> bool {
        !self.is_ok()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub struct DestroyServiceReply {
    pub serial: u32,
    pub result: DestroyServiceResult,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub struct SubscribeServices {
    pub serial: Option<u32>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub struct SubscribeServicesReply {
    pub serial: u32,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub struct ServiceCreatedEvent {
    pub id: ServiceId,
    pub serial: Option<u32>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub struct ServiceDestroyedEvent {
    pub id: ServiceId,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub struct CallFunction {
    pub serial: u32,
    pub service_cookie: ServiceCookie,
    pub function: u32,
    pub args: Value,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub enum CallFunctionResult {
    Ok(Value),
    Err(Value),
    Aborted,
    InvalidService,
    InvalidFunction,
    InvalidArgs,
}

impl CallFunctionResult {
    pub fn is_ok(&self) -> bool {
        match self {
            CallFunctionResult::Ok(_) => true,
            CallFunctionResult::Err(_)
            | CallFunctionResult::Aborted
            | CallFunctionResult::InvalidService
            | CallFunctionResult::InvalidFunction
            | CallFunctionResult::InvalidArgs => false,
        }
    }

    pub fn is_err(&self) -> bool {
        !self.is_ok()
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub struct CallFunctionReply {
    pub serial: u32,
    pub result: CallFunctionResult,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub struct SubscribeEvent {
    pub serial: Option<u32>,
    pub service_cookie: ServiceCookie,
    pub event: u32,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub enum SubscribeEventResult {
    Ok,
    InvalidService,
}

impl SubscribeEventResult {
    pub fn is_ok(&self) -> bool {
        match self {
            SubscribeEventResult::Ok => true,
            SubscribeEventResult::InvalidService => false,
        }
    }

    pub fn is_err(&self) -> bool {
        !self.is_ok()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub struct SubscribeEventReply {
    pub serial: u32,
    pub result: SubscribeEventResult,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub struct UnsubscribeEvent {
    pub service_cookie: ServiceCookie,
    pub event: u32,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub struct EmitEvent {
    pub service_cookie: ServiceCookie,
    pub event: u32,
    pub args: Value,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub struct QueryObject {
    pub serial: u32,
    pub uuid: ObjectUuid,
    pub with_services: bool,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub enum QueryObjectResult {
    Cookie(ObjectCookie),
    Service {
        uuid: ServiceUuid,
        cookie: ServiceCookie,
    },
    Done,
    InvalidObject,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub struct QueryObjectReply {
    pub serial: u32,
    pub result: QueryObjectResult,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub struct QueryServiceVersion {
    pub serial: u32,
    pub cookie: ServiceCookie,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub enum QueryServiceVersionResult {
    Ok(u32),
    InvalidService,
}

impl QueryServiceVersionResult {
    pub fn is_ok(&self) -> bool {
        match self {
            QueryServiceVersionResult::Ok(_) => true,
            QueryServiceVersionResult::InvalidService => false,
        }
    }

    pub fn is_err(&self) -> bool {
        !self.is_ok()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub struct QueryServiceVersionReply {
    pub serial: u32,
    pub result: QueryServiceVersionResult,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub struct CreateChannel {
    pub serial: u32,
    pub claim: ChannelEnd,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub struct CreateChannelReply {
    pub serial: u32,
    pub cookie: ChannelCookie,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub struct DestroyChannelEnd {
    pub serial: u32,
    pub cookie: ChannelCookie,
    pub end: ChannelEnd,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub enum DestroyChannelEndResult {
    Ok,
    InvalidChannel,
    ForeignChannel,
}

impl DestroyChannelEndResult {
    pub fn is_ok(&self) -> bool {
        match self {
            Self::Ok => true,
            Self::InvalidChannel | Self::ForeignChannel => false,
        }
    }

    pub fn is_err(&self) -> bool {
        !self.is_ok()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub struct DestroyChannelEndReply {
    pub serial: u32,
    pub result: DestroyChannelEndResult,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub struct ChannelEndDestroyed {
    pub cookie: ChannelCookie,
    pub end: ChannelEnd,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub struct ClaimChannelEnd {
    pub serial: u32,
    pub cookie: ChannelCookie,
    pub end: ChannelEnd,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub enum ClaimChannelEndResult {
    Ok,
    InvalidChannel,
    AlreadyClaimed,
}

impl ClaimChannelEndResult {
    pub fn is_ok(&self) -> bool {
        match self {
            Self::Ok => true,
            Self::InvalidChannel | Self::AlreadyClaimed => false,
        }
    }

    pub fn is_err(&self) -> bool {
        !self.is_ok()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub struct ClaimChannelEndReply {
    pub serial: u32,
    pub result: ClaimChannelEndResult,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub struct ChannelEndClaimed {
    pub cookie: ChannelCookie,
    pub end: ChannelEnd,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub struct SendItem {
    pub cookie: ChannelCookie,
    pub item: Value,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub struct ItemReceived {
    pub cookie: ChannelCookie,
    pub item: Value,
}
