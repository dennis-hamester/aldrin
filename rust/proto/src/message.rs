use super::Value;
use uuid::Uuid;

#[cfg_attr(feature = "serde-derive", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde-derive",
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
#[derive(Debug, Clone, PartialEq)]
pub enum Message {
    Connect(Connect),
    ConnectReply(ConnectReply),
    Shutdown,
    CreateObject(CreateObject),
    CreateObjectReply(CreateObjectReply),
    DestroyObject(DestroyObject),
    DestroyObjectReply(DestroyObjectReply),
    SubscribeObjects(SubscribeObjects),
    SubscribeObjectsReply(SubscribeObjectsReply),
    UnsubscribeObjects,
    ObjectCreatedEvent(ObjectCreatedEvent),
    ObjectDestroyedEvent(ObjectDestroyedEvent),
    CreateService(CreateService),
    CreateServiceReply(CreateServiceReply),
    DestroyService(DestroyService),
    DestroyServiceReply(DestroyServiceReply),
    SubscribeServices(SubscribeServices),
    SubscribeServicesReply(SubscribeServicesReply),
    UnsubscribeServices,
    ServiceCreatedEvent(ServiceCreatedEvent),
    ServiceDestroyedEvent(ServiceDestroyedEvent),
    CallFunction(CallFunction),
    CallFunctionReply(CallFunctionReply),
    SubscribeEvent(SubscribeEvent),
    SubscribeEventReply(SubscribeEventReply),
    UnsubscribeEvent(UnsubscribeEvent),
    EmitEvent(EmitEvent),
}

#[cfg_attr(feature = "serde-derive", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde-derive",
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Connect {
    pub version: u32,
}

#[cfg_attr(feature = "serde-derive", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde-derive",
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectReply {
    Ok,
    VersionMismatch(u32),
}

#[cfg_attr(feature = "serde-derive", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde-derive",
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateObject {
    pub serial: u32,
    pub uuid: Uuid,
}

#[cfg_attr(feature = "serde-derive", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde-derive",
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
#[derive(Debug, Clone, PartialEq, Eq)]
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

#[cfg_attr(feature = "serde-derive", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde-derive",
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateObjectReply {
    pub serial: u32,
    pub result: CreateObjectResult,
}

#[cfg_attr(feature = "serde-derive", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde-derive",
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DestroyObject {
    pub serial: u32,
    pub cookie: Uuid,
}

#[cfg_attr(feature = "serde-derive", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde-derive",
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
#[derive(Debug, Clone, PartialEq, Eq)]
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

#[cfg_attr(feature = "serde-derive", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde-derive",
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DestroyObjectReply {
    pub serial: u32,
    pub result: DestroyObjectResult,
}

#[cfg_attr(feature = "serde-derive", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde-derive",
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubscribeObjects {
    pub serial: Option<u32>,
}

#[cfg_attr(feature = "serde-derive", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde-derive",
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubscribeObjectsReply {
    pub serial: u32,
}

#[cfg_attr(feature = "serde-derive", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde-derive",
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectCreatedEvent {
    pub uuid: Uuid,
    pub cookie: Uuid,
    pub serial: Option<u32>,
}

#[cfg_attr(feature = "serde-derive", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde-derive",
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectDestroyedEvent {
    pub uuid: Uuid,
    pub cookie: Uuid,
}

#[cfg_attr(feature = "serde-derive", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde-derive",
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateService {
    pub serial: u32,
    pub object_cookie: Uuid,
    pub uuid: Uuid,
}

#[cfg_attr(feature = "serde-derive", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde-derive",
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
#[derive(Debug, Clone, PartialEq, Eq)]
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

#[cfg_attr(feature = "serde-derive", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde-derive",
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateServiceReply {
    pub serial: u32,
    pub result: CreateServiceResult,
}

#[cfg_attr(feature = "serde-derive", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde-derive",
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DestroyService {
    pub serial: u32,
    pub cookie: Uuid,
}

#[cfg_attr(feature = "serde-derive", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde-derive",
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
#[derive(Debug, Clone, PartialEq, Eq)]
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

#[cfg_attr(feature = "serde-derive", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde-derive",
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DestroyServiceReply {
    pub serial: u32,
    pub result: DestroyServiceResult,
}

#[cfg_attr(feature = "serde-derive", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde-derive",
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubscribeServices {
    pub serial: Option<u32>,
}

#[cfg_attr(feature = "serde-derive", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde-derive",
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubscribeServicesReply {
    pub serial: u32,
}

#[cfg_attr(feature = "serde-derive", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde-derive",
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceCreatedEvent {
    pub object_uuid: Uuid,
    pub object_cookie: Uuid,
    pub uuid: Uuid,
    pub cookie: Uuid,
    pub serial: Option<u32>,
}

#[cfg_attr(feature = "serde-derive", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde-derive",
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceDestroyedEvent {
    pub object_uuid: Uuid,
    pub object_cookie: Uuid,
    pub uuid: Uuid,
    pub cookie: Uuid,
}

#[cfg_attr(feature = "serde-derive", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde-derive",
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
#[derive(Debug, Clone, PartialEq)]
pub struct CallFunction {
    pub serial: u32,
    pub service_cookie: Uuid,
    pub function: u32,
    pub args: Value,
}

#[cfg_attr(feature = "serde-derive", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde-derive",
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
#[derive(Debug, Clone, PartialEq)]
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

#[cfg_attr(feature = "serde-derive", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde-derive",
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
#[derive(Debug, Clone, PartialEq)]
pub struct CallFunctionReply {
    pub serial: u32,
    pub result: CallFunctionResult,
}

#[cfg_attr(feature = "serde-derive", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde-derive",
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubscribeEvent {
    pub serial: Option<u32>,
    pub service_cookie: Uuid,
    pub event: u32,
}

#[cfg_attr(feature = "serde-derive", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde-derive",
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
#[derive(Debug, Clone, PartialEq, Eq)]
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

#[cfg_attr(feature = "serde-derive", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde-derive",
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubscribeEventReply {
    pub serial: u32,
    pub result: SubscribeEventResult,
}

#[cfg_attr(feature = "serde-derive", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde-derive",
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnsubscribeEvent {
    pub service_cookie: Uuid,
    pub event: u32,
}

#[cfg_attr(feature = "serde-derive", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde-derive",
    serde(rename_all = "kebab-case", deny_unknown_fields)
)]
#[derive(Debug, Clone, PartialEq)]
pub struct EmitEvent {
    pub service_cookie: Uuid,
    pub event: u32,
    pub args: Value,
}
