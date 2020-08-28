use super::Value;
use uuid::Uuid;

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
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub struct Connect {
    pub version: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub enum ConnectReply {
    Ok,
    VersionMismatch(u32),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub struct CreateObject {
    pub serial: u32,
    pub uuid: Uuid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub enum CreateObjectResult {
    Ok(Uuid),
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

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub struct CreateObjectReply {
    pub serial: u32,
    pub result: CreateObjectResult,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub struct DestroyObject {
    pub serial: u32,
    pub cookie: Uuid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub struct DestroyObjectReply {
    pub serial: u32,
    pub result: DestroyObjectResult,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub struct SubscribeObjects {
    pub serial: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub struct SubscribeObjectsReply {
    pub serial: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub struct ObjectCreatedEvent {
    pub uuid: Uuid,
    pub cookie: Uuid,
    pub serial: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub struct ObjectDestroyedEvent {
    pub uuid: Uuid,
    pub cookie: Uuid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub struct CreateService {
    pub serial: u32,
    pub object_cookie: Uuid,
    pub uuid: Uuid,
    pub version: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub enum CreateServiceResult {
    Ok(Uuid),
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

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub struct CreateServiceReply {
    pub serial: u32,
    pub result: CreateServiceResult,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub struct DestroyService {
    pub serial: u32,
    pub cookie: Uuid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub struct DestroyServiceReply {
    pub serial: u32,
    pub result: DestroyServiceResult,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub struct SubscribeServices {
    pub serial: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub struct SubscribeServicesReply {
    pub serial: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub struct ServiceCreatedEvent {
    pub object_uuid: Uuid,
    pub object_cookie: Uuid,
    pub uuid: Uuid,
    pub cookie: Uuid,
    pub serial: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub struct ServiceDestroyedEvent {
    pub object_uuid: Uuid,
    pub object_cookie: Uuid,
    pub uuid: Uuid,
    pub cookie: Uuid,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub struct CallFunction {
    pub serial: u32,
    pub service_cookie: Uuid,
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

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub struct SubscribeEvent {
    pub serial: Option<u32>,
    pub service_cookie: Uuid,
    pub event: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub struct SubscribeEventReply {
    pub serial: u32,
    pub result: SubscribeEventResult,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub struct UnsubscribeEvent {
    pub service_cookie: Uuid,
    pub event: u32,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub struct EmitEvent {
    pub service_cookie: Uuid,
    pub event: u32,
    pub args: Value,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub struct QueryObject {
    pub serial: u32,
    pub uuid: Uuid,
    pub with_services: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub enum QueryObjectResult {
    Cookie(Uuid),
    Service { uuid: Uuid, cookie: Uuid },
    Done,
    InvalidObject,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub struct QueryObjectReply {
    pub serial: u32,
    pub result: QueryObjectResult,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub struct QueryServiceVersion {
    pub serial: u32,
    pub cookie: Uuid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde-derive",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
pub struct QueryServiceVersionReply {
    pub serial: u32,
    pub result: QueryServiceVersionResult,
}
