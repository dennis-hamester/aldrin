use super::Value;
use uuid::Uuid;

#[cfg_attr(feature = "serde_derive", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub enum Message {
    Connect(Connect),
    ConnectReply(ConnectReply),
    Shutdown,
    CreateObject(CreateObject),
    CreateObjectReply(CreateObjectReply),
    SubscribeObjectsCreated(SubscribeObjectsCreated),
    SubscribeObjectsCreatedReply(SubscribeObjectsCreatedReply),
    UnsubscribeObjectsCreated,
    ObjectCreatedEvent(ObjectCreatedEvent),
    DestroyObject(DestroyObject),
    DestroyObjectReply(DestroyObjectReply),
    SubscribeObjectsDestroyed,
    UnsubscribeObjectsDestroyed,
    ObjectDestroyedEvent(ObjectDestroyedEvent),
    CreateService(CreateService),
    CreateServiceReply(CreateServiceReply),
    SubscribeServicesCreated(SubscribeServicesCreated),
    SubscribeServicesCreatedReply(SubscribeServicesCreatedReply),
    UnsubscribeServicesCreated,
    ServiceCreatedEvent(ServiceCreatedEvent),
    DestroyService(DestroyService),
    DestroyServiceReply(DestroyServiceReply),
    SubscribeServicesDestroyed,
    UnsubscribeServicesDestroyed,
    ServiceDestroyedEvent(ServiceDestroyedEvent),
    CallFunction(CallFunction),
    CallFunctionReply(CallFunctionReply),
    SubscribeEvent(SubscribeEvent),
    SubscribeEventReply(SubscribeEventReply),
    UnsubscribeEvent(UnsubscribeEvent),
    EmitEvent(EmitEvent),
}

#[cfg_attr(feature = "serde_derive", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct Connect {
    pub version: u32,
}

#[cfg_attr(feature = "serde_derive", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub enum ConnectReply {
    Ok,
    VersionMismatch(u32),
}

#[cfg_attr(feature = "serde_derive", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct CreateObject {
    pub serial: u32,
    pub uuid: Uuid,
}

#[cfg_attr(feature = "serde_derive", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
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

#[cfg_attr(feature = "serde_derive", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct CreateObjectReply {
    pub serial: u32,
    pub result: CreateObjectResult,
}

#[cfg_attr(feature = "serde_derive", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct SubscribeObjectsCreated {
    pub serial: Option<u32>,
}

#[cfg_attr(feature = "serde_derive", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct SubscribeObjectsCreatedReply {
    pub serial: u32,
}

#[cfg_attr(feature = "serde_derive", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct ObjectCreatedEvent {
    pub uuid: Uuid,
    pub cookie: Uuid,
    pub serial: Option<u32>,
}

#[cfg_attr(feature = "serde_derive", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct DestroyObject {
    pub serial: u32,
    pub cookie: Uuid,
}

#[cfg_attr(feature = "serde_derive", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
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

#[cfg_attr(feature = "serde_derive", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct DestroyObjectReply {
    pub serial: u32,
    pub result: DestroyObjectResult,
}

#[cfg_attr(feature = "serde_derive", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct ObjectDestroyedEvent {
    pub uuid: Uuid,
    pub cookie: Uuid,
}

#[cfg_attr(feature = "serde_derive", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct CreateService {
    pub serial: u32,
    pub object_cookie: Uuid,
    pub uuid: Uuid,
}

#[cfg_attr(feature = "serde_derive", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
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

#[cfg_attr(feature = "serde_derive", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct CreateServiceReply {
    pub serial: u32,
    pub result: CreateServiceResult,
}

#[cfg_attr(feature = "serde_derive", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct SubscribeServicesCreated {
    pub serial: Option<u32>,
}

#[cfg_attr(feature = "serde_derive", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct SubscribeServicesCreatedReply {
    pub serial: u32,
}

#[cfg_attr(feature = "serde_derive", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct ServiceCreatedEvent {
    pub object_uuid: Uuid,
    pub object_cookie: Uuid,
    pub uuid: Uuid,
    pub cookie: Uuid,
    pub serial: Option<u32>,
}

#[cfg_attr(feature = "serde_derive", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct DestroyService {
    pub serial: u32,
    pub cookie: Uuid,
}

#[cfg_attr(feature = "serde_derive", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
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

#[cfg_attr(feature = "serde_derive", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct DestroyServiceReply {
    pub serial: u32,
    pub result: DestroyServiceResult,
}

#[cfg_attr(feature = "serde_derive", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct ServiceDestroyedEvent {
    pub object_uuid: Uuid,
    pub object_cookie: Uuid,
    pub uuid: Uuid,
    pub cookie: Uuid,
}

#[cfg_attr(feature = "serde_derive", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct CallFunction {
    pub serial: u32,
    pub service_cookie: Uuid,
    pub function: u32,
    pub args: Value,
}

#[cfg_attr(feature = "serde_derive", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
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

#[cfg_attr(feature = "serde_derive", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct CallFunctionReply {
    pub serial: u32,
    pub result: CallFunctionResult,
}

#[cfg_attr(feature = "serde_derive", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct SubscribeEvent {
    pub serial: Option<u32>,
    pub service_cookie: Uuid,
    pub event: u32,
}

#[cfg_attr(feature = "serde_derive", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
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

#[cfg_attr(feature = "serde_derive", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct SubscribeEventReply {
    pub serial: u32,
    pub result: SubscribeEventResult,
}

#[cfg_attr(feature = "serde_derive", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct UnsubscribeEvent {
    pub service_cookie: Uuid,
    pub event: u32,
}

#[cfg_attr(feature = "serde_derive", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct EmitEvent {
    pub service_cookie: Uuid,
    pub event: u32,
    pub args: Value,
}
