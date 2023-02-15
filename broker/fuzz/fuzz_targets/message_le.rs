use crate::context::Context;
use crate::serial_le::SerialLe;
use crate::uuid_le::UuidLe;
use aldrin_proto::message::{
    AddChannelCapacity, CallFunction, CallFunctionReply, CallFunctionResult, ChannelEnd,
    ChannelEndClaimed, ChannelEndClosed, ChannelEndWithCapacity, ClaimChannelEnd,
    ClaimChannelEndReply, ClaimChannelEndResult, CloseChannelEnd, CloseChannelEndReply,
    CloseChannelEndResult, Connect, ConnectReply, CreateChannel, CreateChannelReply, CreateObject,
    CreateObjectReply, CreateObjectResult, CreateService, CreateServiceReply, CreateServiceResult,
    DestroyObject, DestroyObjectReply, DestroyObjectResult, DestroyService, DestroyServiceReply,
    DestroyServiceResult, EmitEvent, ItemReceived, Message as ProtoMessage, ObjectCreatedEvent,
    ObjectDestroyedEvent, QueryObject, QueryObjectReply, QueryObjectResult, QueryServiceVersion,
    QueryServiceVersionReply, QueryServiceVersionResult, SendItem, ServiceCreatedEvent,
    ServiceDestroyedEvent, Shutdown, SubscribeEvent, SubscribeEventReply, SubscribeEventResult,
    SubscribeObjects, SubscribeObjectsReply, SubscribeServices, SubscribeServicesReply, Sync,
    SyncReply, UnsubscribeEvent, UnsubscribeObjects, UnsubscribeServices,
};
use aldrin_proto::{
    ChannelCookie, ObjectCookie, ObjectId, ObjectUuid, ServiceCookie, ServiceId, ServiceUuid,
};
use arbitrary::Arbitrary;

#[derive(Debug, Arbitrary)]
pub enum MessageLe {
    Connect(ConnectLe),
    ConnectReply(ConnectReplyLe),
    Shutdown(ShutdownLe),
    CreateObject(CreateObjectLe),
    CreateObjectReply(CreateObjectReplyLe),
    DestroyObject(DestroyObjectLe),
    DestroyObjectReply(DestroyObjectReplyLe),
    SubscribeObjects(SubscribeObjectsLe),
    SubscribeObjectsReply(SubscribeObjectsReplyLe),
    UnsubscribeObjects(UnsubscribeObjectsLe),
    ObjectCreatedEvent(ObjectCreatedEventLe),
    ObjectDestroyedEvent(ObjectDestroyedEventLe),
    CreateService(CreateServiceLe),
    CreateServiceReply(CreateServiceReplyLe),
    DestroyService(DestroyServiceLe),
    DestroyServiceReply(DestroyServiceReplyLe),
    SubscribeServices(SubscribeServicesLe),
    SubscribeServicesReply(SubscribeServicesReplyLe),
    UnsubscribeServices(UnsubscribeServicesLe),
    ServiceCreatedEvent(ServiceCreatedEventLe),
    ServiceDestroyedEvent(ServiceDestroyedEventLe),
    CallFunction(CallFunctionLe),
    CallFunctionReply(CallFunctionReplyLe),
    SubscribeEvent(SubscribeEventLe),
    SubscribeEventReply(SubscribeEventReplyLe),
    UnsubscribeEvent(UnsubscribeEventLe),
    EmitEvent(EmitEventLe),
    QueryObject(QueryObjectLe),
    QueryObjectReply(QueryObjectReplyLe),
    QueryServiceVersion(QueryServiceVersionLe),
    QueryServiceVersionReply(QueryServiceVersionReplyLe),
    CreateChannel(CreateChannelLe),
    CreateChannelReply(CreateChannelReplyLe),
    CloseChannelEnd(CloseChannelEndLe),
    CloseChannelEndReply(CloseChannelEndReplyLe),
    ChannelEndClosed(ChannelEndClosedLe),
    ClaimChannelEnd(ClaimChannelEndLe),
    ClaimChannelEndReply(ClaimChannelEndReplyLe),
    ChannelEndClaimed(ChannelEndClaimedLe),
    SendItem(SendItemLe),
    ItemReceived(ItemReceivedLe),
    AddChannelCapacity(AddChannelCapacityLe),
    Sync(SyncLe),
    SyncReply(SyncReplyLe),
}

impl MessageLe {
    pub fn to_proto(&self, ctx: &Context) -> ProtoMessage {
        match self {
            Self::Connect(msg) => msg.to_proto(ctx).into(),
            Self::ConnectReply(msg) => msg.to_proto(ctx).into(),
            Self::Shutdown(msg) => msg.to_proto(ctx).into(),
            Self::CreateObject(msg) => msg.to_proto(ctx).into(),
            Self::CreateObjectReply(msg) => msg.to_proto(ctx).into(),
            Self::DestroyObject(msg) => msg.to_proto(ctx).into(),
            Self::DestroyObjectReply(msg) => msg.to_proto(ctx).into(),
            Self::SubscribeObjects(msg) => msg.to_proto(ctx).into(),
            Self::SubscribeObjectsReply(msg) => msg.to_proto(ctx).into(),
            Self::UnsubscribeObjects(msg) => msg.to_proto(ctx).into(),
            Self::ObjectCreatedEvent(msg) => msg.to_proto(ctx).into(),
            Self::ObjectDestroyedEvent(msg) => msg.to_proto(ctx).into(),
            Self::CreateService(msg) => msg.to_proto(ctx).into(),
            Self::CreateServiceReply(msg) => msg.to_proto(ctx).into(),
            Self::DestroyService(msg) => msg.to_proto(ctx).into(),
            Self::DestroyServiceReply(msg) => msg.to_proto(ctx).into(),
            Self::SubscribeServices(msg) => msg.to_proto(ctx).into(),
            Self::SubscribeServicesReply(msg) => msg.to_proto(ctx).into(),
            Self::UnsubscribeServices(msg) => msg.to_proto(ctx).into(),
            Self::ServiceCreatedEvent(msg) => msg.to_proto(ctx).into(),
            Self::ServiceDestroyedEvent(msg) => msg.to_proto(ctx).into(),
            Self::CallFunction(msg) => msg.to_proto(ctx).into(),
            Self::CallFunctionReply(msg) => msg.to_proto(ctx).into(),
            Self::SubscribeEvent(msg) => msg.to_proto(ctx).into(),
            Self::SubscribeEventReply(msg) => msg.to_proto(ctx).into(),
            Self::UnsubscribeEvent(msg) => msg.to_proto(ctx).into(),
            Self::EmitEvent(msg) => msg.to_proto(ctx).into(),
            Self::QueryObject(msg) => msg.to_proto(ctx).into(),
            Self::QueryObjectReply(msg) => msg.to_proto(ctx).into(),
            Self::QueryServiceVersion(msg) => msg.to_proto(ctx).into(),
            Self::QueryServiceVersionReply(msg) => msg.to_proto(ctx).into(),
            Self::CreateChannel(msg) => msg.to_proto(ctx).into(),
            Self::CreateChannelReply(msg) => msg.to_proto(ctx).into(),
            Self::CloseChannelEnd(msg) => msg.to_proto(ctx).into(),
            Self::CloseChannelEndReply(msg) => msg.to_proto(ctx).into(),
            Self::ChannelEndClosed(msg) => msg.to_proto(ctx).into(),
            Self::ClaimChannelEnd(msg) => msg.to_proto(ctx).into(),
            Self::ClaimChannelEndReply(msg) => msg.to_proto(ctx).into(),
            Self::ChannelEndClaimed(msg) => msg.to_proto(ctx).into(),
            Self::SendItem(msg) => msg.to_proto(ctx).into(),
            Self::ItemReceived(msg) => msg.to_proto(ctx).into(),
            Self::AddChannelCapacity(msg) => msg.to_proto(ctx).into(),
            Self::Sync(msg) => msg.to_proto(ctx).into(),
            Self::SyncReply(msg) => msg.to_proto(ctx).into(),
        }
    }
}

pub trait UpdateContext {
    fn update_context(&self, ctx: &mut Context);
}

impl UpdateContext for ProtoMessage {
    fn update_context(&self, ctx: &mut Context) {
        match self {
            Self::Connect(msg) => msg.update_context(ctx),
            Self::ConnectReply(msg) => msg.update_context(ctx),
            Self::Shutdown(msg) => msg.update_context(ctx),
            Self::CreateObject(msg) => msg.update_context(ctx),
            Self::CreateObjectReply(msg) => msg.update_context(ctx),
            Self::DestroyObject(msg) => msg.update_context(ctx),
            Self::DestroyObjectReply(msg) => msg.update_context(ctx),
            Self::SubscribeObjects(msg) => msg.update_context(ctx),
            Self::SubscribeObjectsReply(msg) => msg.update_context(ctx),
            Self::UnsubscribeObjects(msg) => msg.update_context(ctx),
            Self::ObjectCreatedEvent(msg) => msg.update_context(ctx),
            Self::ObjectDestroyedEvent(msg) => msg.update_context(ctx),
            Self::CreateService(msg) => msg.update_context(ctx),
            Self::CreateServiceReply(msg) => msg.update_context(ctx),
            Self::DestroyService(msg) => msg.update_context(ctx),
            Self::DestroyServiceReply(msg) => msg.update_context(ctx),
            Self::SubscribeServices(msg) => msg.update_context(ctx),
            Self::SubscribeServicesReply(msg) => msg.update_context(ctx),
            Self::UnsubscribeServices(msg) => msg.update_context(ctx),
            Self::ServiceCreatedEvent(msg) => msg.update_context(ctx),
            Self::ServiceDestroyedEvent(msg) => msg.update_context(ctx),
            Self::CallFunction(msg) => msg.update_context(ctx),
            Self::CallFunctionReply(msg) => msg.update_context(ctx),
            Self::SubscribeEvent(msg) => msg.update_context(ctx),
            Self::SubscribeEventReply(msg) => msg.update_context(ctx),
            Self::UnsubscribeEvent(msg) => msg.update_context(ctx),
            Self::EmitEvent(msg) => msg.update_context(ctx),
            Self::QueryObject(msg) => msg.update_context(ctx),
            Self::QueryObjectReply(msg) => msg.update_context(ctx),
            Self::QueryServiceVersion(msg) => msg.update_context(ctx),
            Self::QueryServiceVersionReply(msg) => msg.update_context(ctx),
            Self::CreateChannel(msg) => msg.update_context(ctx),
            Self::CreateChannelReply(msg) => msg.update_context(ctx),
            Self::CloseChannelEnd(msg) => msg.update_context(ctx),
            Self::CloseChannelEndReply(msg) => msg.update_context(ctx),
            Self::ChannelEndClosed(msg) => msg.update_context(ctx),
            Self::ClaimChannelEnd(msg) => msg.update_context(ctx),
            Self::ClaimChannelEndReply(msg) => msg.update_context(ctx),
            Self::ChannelEndClaimed(msg) => msg.update_context(ctx),
            Self::SendItem(msg) => msg.update_context(ctx),
            Self::ItemReceived(msg) => msg.update_context(ctx),
            Self::AddChannelCapacity(msg) => msg.update_context(ctx),
            Self::Sync(msg) => msg.update_context(ctx),
            Self::SyncReply(msg) => msg.update_context(ctx),
        }
    }
}

#[derive(Debug, Arbitrary)]
pub struct ConnectLe {
    pub version: u8,
}

impl ConnectLe {
    pub fn to_proto(&self, _ctx: &Context) -> Connect {
        Connect::with_serialize_value(self.version as u32, &()).unwrap()
    }
}

impl UpdateContext for Connect {
    fn update_context(&self, _ctx: &mut Context) {}
}

#[derive(Debug, Arbitrary)]
pub enum ConnectReplyLe {
    Ok,
    VersionMismatch(u8),
    Rejected,
}

impl ConnectReplyLe {
    pub fn to_proto(&self, _ctx: &Context) -> ConnectReply {
        match self {
            Self::Ok => ConnectReply::ok_with_serialize_value(&()).unwrap(),
            Self::VersionMismatch(version) => ConnectReply::VersionMismatch(*version as u32),
            Self::Rejected => ConnectReply::rejected_with_serialize_value(&()).unwrap(),
        }
    }
}

impl UpdateContext for ConnectReply {
    fn update_context(&self, _ctx: &mut Context) {}
}

#[derive(Debug, Arbitrary)]
pub struct ShutdownLe;

impl ShutdownLe {
    pub fn to_proto(&self, _ctx: &Context) -> Shutdown {
        Shutdown
    }
}

impl UpdateContext for Shutdown {
    fn update_context(&self, _ctx: &mut Context) {}
}

#[derive(Debug, Arbitrary)]
pub struct CreateObjectLe {
    pub serial: SerialLe,
    pub uuid: UuidLe,
}

impl CreateObjectLe {
    pub fn to_proto(&self, ctx: &Context) -> CreateObject {
        CreateObject {
            serial: self.serial.get(ctx),
            uuid: ObjectUuid(self.uuid.get(ctx)),
        }
    }
}

impl UpdateContext for CreateObject {
    fn update_context(&self, ctx: &mut Context) {
        ctx.add_serial(self.serial);
        ctx.add_uuid(self.uuid.0);
    }
}

#[derive(Debug, Arbitrary)]
pub enum CreateObjectResultLe {
    Ok(UuidLe),
    DuplicateObject,
}

impl CreateObjectResultLe {
    pub fn to_proto(&self, ctx: &Context) -> CreateObjectResult {
        match self {
            Self::Ok(uuid) => CreateObjectResult::Ok(ObjectCookie(uuid.get(ctx))),
            Self::DuplicateObject => CreateObjectResult::DuplicateObject,
        }
    }
}

impl UpdateContext for CreateObjectResult {
    fn update_context(&self, ctx: &mut Context) {
        if let Self::Ok(uuid) = self {
            ctx.add_uuid(uuid.0);
        }
    }
}

#[derive(Debug, Arbitrary)]
pub struct CreateObjectReplyLe {
    pub serial: SerialLe,
    pub result: CreateObjectResultLe,
}

impl CreateObjectReplyLe {
    pub fn to_proto(&self, ctx: &Context) -> CreateObjectReply {
        CreateObjectReply {
            serial: self.serial.get(ctx),
            result: self.result.to_proto(ctx),
        }
    }
}

impl UpdateContext for CreateObjectReply {
    fn update_context(&self, ctx: &mut Context) {
        ctx.add_serial(self.serial);
        self.result.update_context(ctx);
    }
}

#[derive(Debug, Arbitrary)]
pub struct DestroyObjectLe {
    pub serial: SerialLe,
    pub cookie: UuidLe,
}

impl DestroyObjectLe {
    pub fn to_proto(&self, ctx: &Context) -> DestroyObject {
        DestroyObject {
            serial: self.serial.get(ctx),
            cookie: ObjectCookie(self.cookie.get(ctx)),
        }
    }
}

impl UpdateContext for DestroyObject {
    fn update_context(&self, ctx: &mut Context) {
        ctx.add_serial(self.serial);
        ctx.add_uuid(self.cookie.0);
    }
}

#[derive(Debug, Arbitrary)]
pub enum DestroyObjectResultLe {
    Ok,
    InvalidObject,
    ForeignObject,
}

impl DestroyObjectResultLe {
    pub fn to_proto(&self, _ctx: &Context) -> DestroyObjectResult {
        match self {
            Self::Ok => DestroyObjectResult::Ok,
            Self::InvalidObject => DestroyObjectResult::InvalidObject,
            Self::ForeignObject => DestroyObjectResult::ForeignObject,
        }
    }
}

impl UpdateContext for DestroyObjectResult {
    fn update_context(&self, _ctx: &mut Context) {}
}

#[derive(Debug, Arbitrary)]
pub struct DestroyObjectReplyLe {
    pub serial: SerialLe,
    pub result: DestroyObjectResultLe,
}

impl DestroyObjectReplyLe {
    pub fn to_proto(&self, ctx: &Context) -> DestroyObjectReply {
        DestroyObjectReply {
            serial: self.serial.get(ctx),
            result: self.result.to_proto(ctx),
        }
    }
}

impl UpdateContext for DestroyObjectReply {
    fn update_context(&self, ctx: &mut Context) {
        ctx.add_serial(self.serial);
        self.result.update_context(ctx);
    }
}

#[derive(Debug, Arbitrary)]
pub struct SubscribeObjectsLe {
    pub serial: Option<SerialLe>,
}

impl SubscribeObjectsLe {
    pub fn to_proto(&self, ctx: &Context) -> SubscribeObjects {
        SubscribeObjects {
            serial: self.serial.as_ref().map(|serial| serial.get(ctx)),
        }
    }
}

impl UpdateContext for SubscribeObjects {
    fn update_context(&self, ctx: &mut Context) {
        if let Some(serial) = self.serial {
            ctx.add_serial(serial);
        }
    }
}

#[derive(Debug, Arbitrary)]
pub struct SubscribeObjectsReplyLe {
    pub serial: SerialLe,
}

impl SubscribeObjectsReplyLe {
    pub fn to_proto(&self, ctx: &Context) -> SubscribeObjectsReply {
        SubscribeObjectsReply {
            serial: self.serial.get(ctx),
        }
    }
}

impl UpdateContext for SubscribeObjectsReply {
    fn update_context(&self, ctx: &mut Context) {
        ctx.add_serial(self.serial);
    }
}

#[derive(Debug, Arbitrary)]
pub struct UnsubscribeObjectsLe;

impl UnsubscribeObjectsLe {
    pub fn to_proto(&self, _ctx: &Context) -> UnsubscribeObjects {
        UnsubscribeObjects
    }
}

impl UpdateContext for UnsubscribeObjects {
    fn update_context(&self, _ctx: &mut Context) {}
}

#[derive(Debug, Arbitrary)]
pub struct ObjectCreatedEventLe {
    pub uuid: UuidLe,
    pub cookie: UuidLe,
    pub serial: Option<SerialLe>,
}

impl ObjectCreatedEventLe {
    pub fn to_proto(&self, ctx: &Context) -> ObjectCreatedEvent {
        ObjectCreatedEvent {
            id: ObjectId {
                uuid: ObjectUuid(self.uuid.get(ctx)),
                cookie: ObjectCookie(self.cookie.get(ctx)),
            },
            serial: self.serial.as_ref().map(|serial| serial.get(ctx)),
        }
    }
}

impl UpdateContext for ObjectCreatedEvent {
    fn update_context(&self, ctx: &mut Context) {
        ctx.add_uuid(self.id.uuid.0);
        ctx.add_uuid(self.id.cookie.0);
        if let Some(serial) = self.serial {
            ctx.add_serial(serial);
        }
    }
}

#[derive(Debug, Arbitrary)]
pub struct ObjectDestroyedEventLe {
    pub uuid: UuidLe,
    pub cookie: UuidLe,
}

impl ObjectDestroyedEventLe {
    pub fn to_proto(&self, ctx: &Context) -> ObjectDestroyedEvent {
        ObjectDestroyedEvent {
            id: ObjectId {
                uuid: ObjectUuid(self.uuid.get(ctx)),
                cookie: ObjectCookie(self.cookie.get(ctx)),
            },
        }
    }
}

impl UpdateContext for ObjectDestroyedEvent {
    fn update_context(&self, ctx: &mut Context) {
        ctx.add_uuid(self.id.uuid.0);
        ctx.add_uuid(self.id.cookie.0);
    }
}

#[derive(Debug, Arbitrary)]
pub struct CreateServiceLe {
    pub serial: SerialLe,
    pub object_cookie: UuidLe,
    pub uuid: UuidLe,
    pub version: u8,
}

impl CreateServiceLe {
    pub fn to_proto(&self, ctx: &Context) -> CreateService {
        CreateService {
            serial: self.serial.get(ctx),
            object_cookie: ObjectCookie(self.object_cookie.get(ctx)),
            uuid: ServiceUuid(self.uuid.get(ctx)),
            version: self.version as u32,
        }
    }
}

impl UpdateContext for CreateService {
    fn update_context(&self, ctx: &mut Context) {
        ctx.add_serial(self.serial);
        ctx.add_uuid(self.object_cookie.0);
        ctx.add_uuid(self.uuid.0);
    }
}

#[derive(Debug, Arbitrary)]
pub enum CreateServiceResultLe {
    Ok(UuidLe),
    DuplicateService,
    InvalidObject,
    ForeignObject,
}

impl CreateServiceResultLe {
    pub fn to_proto(&self, ctx: &Context) -> CreateServiceResult {
        match self {
            Self::Ok(uuid) => CreateServiceResult::Ok(ServiceCookie(uuid.get(ctx))),
            Self::DuplicateService => CreateServiceResult::DuplicateService,
            Self::InvalidObject => CreateServiceResult::InvalidObject,
            Self::ForeignObject => CreateServiceResult::ForeignObject,
        }
    }
}

impl UpdateContext for CreateServiceResult {
    fn update_context(&self, ctx: &mut Context) {
        if let Self::Ok(uuid) = self {
            ctx.add_uuid(uuid.0);
        }
    }
}

#[derive(Debug, Arbitrary)]
pub struct CreateServiceReplyLe {
    pub serial: SerialLe,
    pub result: CreateServiceResultLe,
}

impl CreateServiceReplyLe {
    pub fn to_proto(&self, ctx: &Context) -> CreateServiceReply {
        CreateServiceReply {
            serial: self.serial.get(ctx),
            result: self.result.to_proto(ctx),
        }
    }
}

impl UpdateContext for CreateServiceReply {
    fn update_context(&self, ctx: &mut Context) {
        ctx.add_serial(self.serial);
        self.result.update_context(ctx);
    }
}

#[derive(Debug, Arbitrary)]
pub struct DestroyServiceLe {
    pub serial: SerialLe,
    pub cookie: UuidLe,
}

impl DestroyServiceLe {
    pub fn to_proto(&self, ctx: &Context) -> DestroyService {
        DestroyService {
            serial: self.serial.get(ctx),
            cookie: ServiceCookie(self.cookie.get(ctx)),
        }
    }
}

impl UpdateContext for DestroyService {
    fn update_context(&self, ctx: &mut Context) {
        ctx.add_serial(self.serial);
        ctx.add_uuid(self.cookie.0);
    }
}

#[derive(Debug, Arbitrary)]
pub enum DestroyServiceResultLe {
    Ok,
    InvalidService,
    ForeignObject,
}

impl DestroyServiceResultLe {
    pub fn to_proto(&self, _ctx: &Context) -> DestroyServiceResult {
        match self {
            Self::Ok => DestroyServiceResult::Ok,
            Self::InvalidService => DestroyServiceResult::InvalidService,
            Self::ForeignObject => DestroyServiceResult::ForeignObject,
        }
    }
}

impl UpdateContext for DestroyServiceResult {
    fn update_context(&self, _ctx: &mut Context) {}
}

#[derive(Debug, Arbitrary)]
pub struct DestroyServiceReplyLe {
    pub serial: SerialLe,
    pub result: DestroyServiceResultLe,
}

impl DestroyServiceReplyLe {
    pub fn to_proto(&self, ctx: &Context) -> DestroyServiceReply {
        DestroyServiceReply {
            serial: self.serial.get(ctx),
            result: self.result.to_proto(ctx),
        }
    }
}

impl UpdateContext for DestroyServiceReply {
    fn update_context(&self, ctx: &mut Context) {
        ctx.add_serial(self.serial);
        self.result.update_context(ctx);
    }
}

#[derive(Debug, Arbitrary)]
pub struct SubscribeServicesLe {
    pub serial: Option<SerialLe>,
}

impl SubscribeServicesLe {
    pub fn to_proto(&self, ctx: &Context) -> SubscribeServices {
        SubscribeServices {
            serial: self.serial.as_ref().map(|serial| serial.get(ctx)),
        }
    }
}

impl UpdateContext for SubscribeServices {
    fn update_context(&self, ctx: &mut Context) {
        if let Some(serial) = self.serial {
            ctx.add_serial(serial);
        }
    }
}

#[derive(Debug, Arbitrary)]
pub struct SubscribeServicesReplyLe {
    pub serial: SerialLe,
}

impl SubscribeServicesReplyLe {
    pub fn to_proto(&self, ctx: &Context) -> SubscribeServicesReply {
        SubscribeServicesReply {
            serial: self.serial.get(ctx),
        }
    }
}

impl UpdateContext for SubscribeServicesReply {
    fn update_context(&self, ctx: &mut Context) {
        ctx.add_serial(self.serial);
    }
}

#[derive(Debug, Arbitrary)]
pub struct UnsubscribeServicesLe;

impl UnsubscribeServicesLe {
    pub fn to_proto(&self, _ctx: &Context) -> UnsubscribeServices {
        UnsubscribeServices
    }
}

impl UpdateContext for UnsubscribeServices {
    fn update_context(&self, _ctx: &mut Context) {}
}

#[derive(Debug, Arbitrary)]
pub struct ServiceCreatedEventLe {
    pub object_uuid: UuidLe,
    pub object_cookie: UuidLe,
    pub service_uuid: UuidLe,
    pub service_cookie: UuidLe,
    pub serial: Option<SerialLe>,
}

impl ServiceCreatedEventLe {
    pub fn to_proto(&self, ctx: &Context) -> ServiceCreatedEvent {
        ServiceCreatedEvent {
            id: ServiceId {
                object_id: ObjectId {
                    uuid: ObjectUuid(self.object_uuid.get(ctx)),
                    cookie: ObjectCookie(self.object_cookie.get(ctx)),
                },
                uuid: ServiceUuid(self.service_uuid.get(ctx)),
                cookie: ServiceCookie(self.service_cookie.get(ctx)),
            },
            serial: self.serial.as_ref().map(|serial| serial.get(ctx)),
        }
    }
}

impl UpdateContext for ServiceCreatedEvent {
    fn update_context(&self, ctx: &mut Context) {
        ctx.add_uuid(self.id.object_id.uuid.0);
        ctx.add_uuid(self.id.object_id.cookie.0);
        ctx.add_uuid(self.id.uuid.0);
        ctx.add_uuid(self.id.cookie.0);
        if let Some(serial) = self.serial {
            ctx.add_serial(serial);
        }
    }
}

#[derive(Debug, Arbitrary)]
pub struct ServiceDestroyedEventLe {
    pub object_uuid: UuidLe,
    pub object_cookie: UuidLe,
    pub service_uuid: UuidLe,
    pub service_cookie: UuidLe,
}

impl ServiceDestroyedEventLe {
    pub fn to_proto(&self, ctx: &Context) -> ServiceDestroyedEvent {
        ServiceDestroyedEvent {
            id: ServiceId {
                object_id: ObjectId {
                    uuid: ObjectUuid(self.object_uuid.get(ctx)),
                    cookie: ObjectCookie(self.object_cookie.get(ctx)),
                },
                uuid: ServiceUuid(self.service_uuid.get(ctx)),
                cookie: ServiceCookie(self.service_cookie.get(ctx)),
            },
        }
    }
}

impl UpdateContext for ServiceDestroyedEvent {
    fn update_context(&self, ctx: &mut Context) {
        ctx.add_uuid(self.id.object_id.uuid.0);
        ctx.add_uuid(self.id.object_id.cookie.0);
        ctx.add_uuid(self.id.uuid.0);
        ctx.add_uuid(self.id.cookie.0);
    }
}

#[derive(Debug, Arbitrary)]
pub struct CallFunctionLe {
    pub serial: SerialLe,
    pub service_cookie: UuidLe,
    pub function: u8,
}

impl CallFunctionLe {
    pub fn to_proto(&self, ctx: &Context) -> CallFunction {
        CallFunction::with_serialize_value(
            self.serial.get(ctx),
            ServiceCookie(self.service_cookie.get(ctx)),
            self.function as u32,
            &(),
        )
        .unwrap()
    }
}

impl UpdateContext for CallFunction {
    fn update_context(&self, ctx: &mut Context) {
        ctx.add_serial(self.serial);
        ctx.add_uuid(self.service_cookie.0);
    }
}

#[derive(Debug, Arbitrary)]
pub enum CallFunctionResultLe {
    Ok,
    Err,
    Aborted,
    InvalidService,
    InvalidFunction,
    InvalidArgs,
}

impl CallFunctionResultLe {
    pub fn to_proto(&self, _ctx: &Context) -> CallFunctionResult {
        match self {
            Self::Ok => CallFunctionResult::ok_with_serialize_value(&()).unwrap(),
            Self::Err => CallFunctionResult::err_with_serialize_value(&()).unwrap(),
            Self::Aborted => CallFunctionResult::Aborted,
            Self::InvalidService => CallFunctionResult::InvalidService,
            Self::InvalidFunction => CallFunctionResult::InvalidFunction,
            Self::InvalidArgs => CallFunctionResult::InvalidArgs,
        }
    }
}

impl UpdateContext for CallFunctionResult {
    fn update_context(&self, _ctx: &mut Context) {}
}

#[derive(Debug, Arbitrary)]
pub struct CallFunctionReplyLe {
    pub serial: SerialLe,
    pub result: CallFunctionResultLe,
}

impl CallFunctionReplyLe {
    pub fn to_proto(&self, ctx: &Context) -> CallFunctionReply {
        CallFunctionReply {
            serial: self.serial.get(ctx),
            result: self.result.to_proto(ctx),
        }
    }
}

impl UpdateContext for CallFunctionReply {
    fn update_context(&self, ctx: &mut Context) {
        ctx.add_serial(self.serial);
        self.result.update_context(ctx);
    }
}

#[derive(Debug, Arbitrary)]
pub struct SubscribeEventLe {
    pub serial: Option<SerialLe>,
    pub service_cookie: UuidLe,
    pub event: u8,
}

impl SubscribeEventLe {
    pub fn to_proto(&self, ctx: &Context) -> SubscribeEvent {
        SubscribeEvent {
            serial: self.serial.as_ref().map(|serial| serial.get(ctx)),
            service_cookie: ServiceCookie(self.service_cookie.get(ctx)),
            event: self.event as u32,
        }
    }
}

impl UpdateContext for SubscribeEvent {
    fn update_context(&self, ctx: &mut Context) {
        if let Some(serial) = self.serial {
            ctx.add_serial(serial);
        }
        ctx.add_uuid(self.service_cookie.0);
    }
}

#[derive(Debug, Arbitrary)]
pub enum SubscribeEventResultLe {
    Ok,
    InvalidService,
}

impl SubscribeEventResultLe {
    pub fn to_proto(&self, _ctx: &Context) -> SubscribeEventResult {
        match self {
            Self::Ok => SubscribeEventResult::Ok,
            Self::InvalidService => SubscribeEventResult::InvalidService,
        }
    }
}

impl UpdateContext for SubscribeEventResult {
    fn update_context(&self, _ctx: &mut Context) {}
}

#[derive(Debug, Arbitrary)]
pub struct SubscribeEventReplyLe {
    pub serial: SerialLe,
    pub result: SubscribeEventResultLe,
}

impl SubscribeEventReplyLe {
    pub fn to_proto(&self, ctx: &Context) -> SubscribeEventReply {
        SubscribeEventReply {
            serial: self.serial.get(ctx),
            result: self.result.to_proto(ctx),
        }
    }
}

impl UpdateContext for SubscribeEventReply {
    fn update_context(&self, ctx: &mut Context) {
        ctx.add_serial(self.serial);
        self.result.update_context(ctx);
    }
}

#[derive(Debug, Arbitrary)]
pub struct UnsubscribeEventLe {
    pub service_cookie: UuidLe,
    pub event: u8,
}

impl UnsubscribeEventLe {
    pub fn to_proto(&self, ctx: &Context) -> UnsubscribeEvent {
        UnsubscribeEvent {
            service_cookie: ServiceCookie(self.service_cookie.get(ctx)),
            event: self.event as u32,
        }
    }
}

impl UpdateContext for UnsubscribeEvent {
    fn update_context(&self, ctx: &mut Context) {
        ctx.add_uuid(self.service_cookie.0);
    }
}

#[derive(Debug, Arbitrary)]
pub struct EmitEventLe {
    pub service_cookie: UuidLe,
    pub event: u8,
}

impl EmitEventLe {
    pub fn to_proto(&self, ctx: &Context) -> EmitEvent {
        EmitEvent::with_serialize_value(
            ServiceCookie(self.service_cookie.get(ctx)),
            self.event as u32,
            &(),
        )
        .unwrap()
    }
}

impl UpdateContext for EmitEvent {
    fn update_context(&self, ctx: &mut Context) {
        ctx.add_uuid(self.service_cookie.0);
    }
}

#[derive(Debug, Arbitrary)]
pub struct QueryObjectLe {
    pub serial: SerialLe,
    pub uuid: UuidLe,
    pub with_services: bool,
}

impl QueryObjectLe {
    pub fn to_proto(&self, ctx: &Context) -> QueryObject {
        QueryObject {
            serial: self.serial.get(ctx),
            uuid: ObjectUuid(self.uuid.get(ctx)),
            with_services: self.with_services,
        }
    }
}

impl UpdateContext for QueryObject {
    fn update_context(&self, ctx: &mut Context) {
        ctx.add_serial(self.serial);
        ctx.add_uuid(self.uuid.0);
    }
}

#[derive(Debug, Arbitrary)]
pub enum QueryObjectResultLe {
    Cookie(UuidLe),
    Service { uuid: UuidLe, cookie: UuidLe },
    Done,
    InvalidObject,
}

impl QueryObjectResultLe {
    pub fn to_proto(&self, ctx: &Context) -> QueryObjectResult {
        match self {
            Self::Cookie(cookie) => QueryObjectResult::Cookie(ObjectCookie(cookie.get(ctx))),
            Self::Service { uuid, cookie } => QueryObjectResult::Service {
                uuid: ServiceUuid(uuid.get(ctx)),
                cookie: ServiceCookie(cookie.get(ctx)),
            },
            Self::Done => QueryObjectResult::Done,
            Self::InvalidObject => QueryObjectResult::InvalidObject,
        }
    }
}

impl UpdateContext for QueryObjectResult {
    fn update_context(&self, ctx: &mut Context) {
        match self {
            Self::Cookie(cookie) => ctx.add_uuid(cookie.0),
            Self::Service { uuid, cookie } => {
                ctx.add_uuid(uuid.0);
                ctx.add_uuid(cookie.0);
            }
            Self::Done | Self::InvalidObject => {}
        }
    }
}

#[derive(Debug, Arbitrary)]
pub struct QueryObjectReplyLe {
    pub serial: SerialLe,
    pub result: QueryObjectResultLe,
}

impl QueryObjectReplyLe {
    pub fn to_proto(&self, ctx: &Context) -> QueryObjectReply {
        QueryObjectReply {
            serial: self.serial.get(ctx),
            result: self.result.to_proto(ctx),
        }
    }
}

impl UpdateContext for QueryObjectReply {
    fn update_context(&self, ctx: &mut Context) {
        ctx.add_serial(self.serial);
        self.result.update_context(ctx);
    }
}

#[derive(Debug, Arbitrary)]
pub struct QueryServiceVersionLe {
    pub serial: SerialLe,
    pub cookie: UuidLe,
}

impl QueryServiceVersionLe {
    pub fn to_proto(&self, ctx: &Context) -> QueryServiceVersion {
        QueryServiceVersion {
            serial: self.serial.get(ctx),
            cookie: ServiceCookie(self.cookie.get(ctx)),
        }
    }
}

impl UpdateContext for QueryServiceVersion {
    fn update_context(&self, ctx: &mut Context) {
        ctx.add_serial(self.serial);
        ctx.add_uuid(self.cookie.0);
    }
}

#[derive(Debug, Arbitrary)]
pub enum QueryServiceVersionResultLe {
    Ok(u8),
    InvalidService,
}

impl QueryServiceVersionResultLe {
    pub fn to_proto(&self, _ctx: &Context) -> QueryServiceVersionResult {
        match self {
            Self::Ok(version) => QueryServiceVersionResult::Ok(*version as u32),
            Self::InvalidService => QueryServiceVersionResult::InvalidService,
        }
    }
}

impl UpdateContext for QueryServiceVersionResult {
    fn update_context(&self, _ctx: &mut Context) {}
}

#[derive(Debug, Arbitrary)]
pub struct QueryServiceVersionReplyLe {
    pub serial: SerialLe,
    pub result: QueryServiceVersionResultLe,
}

impl QueryServiceVersionReplyLe {
    pub fn to_proto(&self, ctx: &Context) -> QueryServiceVersionReply {
        QueryServiceVersionReply {
            serial: self.serial.get(ctx),
            result: self.result.to_proto(ctx),
        }
    }
}

impl UpdateContext for QueryServiceVersionReply {
    fn update_context(&self, ctx: &mut Context) {
        ctx.add_serial(self.serial);
        self.result.update_context(ctx);
    }
}

#[derive(Debug, Arbitrary)]
pub struct CreateChannelLe {
    pub serial: SerialLe,
    pub end: ChannelEndWithCapacity,
}

impl CreateChannelLe {
    pub fn to_proto(&self, ctx: &Context) -> CreateChannel {
        CreateChannel {
            serial: self.serial.get(ctx),
            end: self.end,
        }
    }
}

impl UpdateContext for CreateChannel {
    fn update_context(&self, ctx: &mut Context) {
        ctx.add_serial(self.serial);
    }
}

#[derive(Debug, Arbitrary)]
pub struct CreateChannelReplyLe {
    pub serial: SerialLe,
    pub cookie: UuidLe,
}

impl CreateChannelReplyLe {
    pub fn to_proto(&self, ctx: &Context) -> CreateChannelReply {
        CreateChannelReply {
            serial: self.serial.get(ctx),
            cookie: ChannelCookie(self.cookie.get(ctx)),
        }
    }
}

impl UpdateContext for CreateChannelReply {
    fn update_context(&self, ctx: &mut Context) {
        ctx.add_serial(self.serial);
        ctx.add_uuid(self.cookie.0);
    }
}

#[derive(Debug, Arbitrary)]
pub struct CloseChannelEndLe {
    pub serial: SerialLe,
    pub cookie: UuidLe,
    pub end: ChannelEnd,
}

impl CloseChannelEndLe {
    pub fn to_proto(&self, ctx: &Context) -> CloseChannelEnd {
        CloseChannelEnd {
            serial: self.serial.get(ctx),
            cookie: ChannelCookie(self.cookie.get(ctx)),
            end: self.end,
        }
    }
}

impl UpdateContext for CloseChannelEnd {
    fn update_context(&self, ctx: &mut Context) {
        ctx.add_serial(self.serial);
        ctx.add_uuid(self.cookie.0);
    }
}

#[derive(Debug, Arbitrary)]
pub enum CloseChannelEndResultLe {
    Ok,
    InvalidChannel,
    ForeignChannel,
}

impl CloseChannelEndResultLe {
    pub fn to_proto(&self, _ctx: &Context) -> CloseChannelEndResult {
        match self {
            Self::Ok => CloseChannelEndResult::Ok,
            Self::InvalidChannel => CloseChannelEndResult::InvalidChannel,
            Self::ForeignChannel => CloseChannelEndResult::ForeignChannel,
        }
    }
}

impl UpdateContext for CloseChannelEndResult {
    fn update_context(&self, _ctx: &mut Context) {}
}

#[derive(Debug, Arbitrary)]
pub struct CloseChannelEndReplyLe {
    pub serial: SerialLe,
    pub result: CloseChannelEndResultLe,
}

impl CloseChannelEndReplyLe {
    pub fn to_proto(&self, ctx: &Context) -> CloseChannelEndReply {
        CloseChannelEndReply {
            serial: self.serial.get(ctx),
            result: self.result.to_proto(ctx),
        }
    }
}

impl UpdateContext for CloseChannelEndReply {
    fn update_context(&self, ctx: &mut Context) {
        ctx.add_serial(self.serial);
        self.result.update_context(ctx);
    }
}

#[derive(Debug, Arbitrary)]
pub struct ChannelEndClosedLe {
    pub cookie: UuidLe,
    pub end: ChannelEnd,
}

impl ChannelEndClosedLe {
    pub fn to_proto(&self, ctx: &Context) -> ChannelEndClosed {
        ChannelEndClosed {
            cookie: ChannelCookie(self.cookie.get(ctx)),
            end: self.end,
        }
    }
}

impl UpdateContext for ChannelEndClosed {
    fn update_context(&self, ctx: &mut Context) {
        ctx.add_uuid(self.cookie.0);
    }
}

#[derive(Debug, Arbitrary)]
pub struct ClaimChannelEndLe {
    pub serial: SerialLe,
    pub cookie: UuidLe,
    pub end: ChannelEndWithCapacity,
}

impl ClaimChannelEndLe {
    pub fn to_proto(&self, ctx: &Context) -> ClaimChannelEnd {
        ClaimChannelEnd {
            serial: self.serial.get(ctx),
            cookie: ChannelCookie(self.cookie.get(ctx)),
            end: self.end,
        }
    }
}

impl UpdateContext for ClaimChannelEnd {
    fn update_context(&self, ctx: &mut Context) {
        ctx.add_serial(self.serial);
        ctx.add_uuid(self.cookie.0);
    }
}

#[derive(Debug, Arbitrary)]
pub enum ClaimChannelEndResultLe {
    SenderClaimed(u32),
    ReceiverClaimed,
    InvalidChannel,
    AlreadyClaimed,
}

impl ClaimChannelEndResultLe {
    pub fn to_proto(&self, _ctx: &Context) -> ClaimChannelEndResult {
        match self {
            Self::SenderClaimed(capacity) => ClaimChannelEndResult::SenderClaimed(*capacity),
            Self::ReceiverClaimed => ClaimChannelEndResult::ReceiverClaimed,
            Self::InvalidChannel => ClaimChannelEndResult::InvalidChannel,
            Self::AlreadyClaimed => ClaimChannelEndResult::AlreadyClaimed,
        }
    }
}

impl UpdateContext for ClaimChannelEndResult {
    fn update_context(&self, _ctx: &mut Context) {}
}

#[derive(Debug, Arbitrary)]
pub struct ClaimChannelEndReplyLe {
    pub serial: SerialLe,
    pub result: ClaimChannelEndResultLe,
}

impl ClaimChannelEndReplyLe {
    pub fn to_proto(&self, ctx: &Context) -> ClaimChannelEndReply {
        ClaimChannelEndReply {
            serial: self.serial.get(ctx),
            result: self.result.to_proto(ctx),
        }
    }
}

impl UpdateContext for ClaimChannelEndReply {
    fn update_context(&self, ctx: &mut Context) {
        ctx.add_serial(self.serial);
        self.result.update_context(ctx);
    }
}

#[derive(Debug, Arbitrary)]
pub struct ChannelEndClaimedLe {
    pub cookie: UuidLe,
    pub end: ChannelEndWithCapacity,
}

impl ChannelEndClaimedLe {
    pub fn to_proto(&self, ctx: &Context) -> ChannelEndClaimed {
        ChannelEndClaimed {
            cookie: ChannelCookie(self.cookie.get(ctx)),
            end: self.end,
        }
    }
}

impl UpdateContext for ChannelEndClaimed {
    fn update_context(&self, ctx: &mut Context) {
        ctx.add_uuid(self.cookie.0);
    }
}

#[derive(Debug, Arbitrary)]
pub struct SendItemLe {
    pub cookie: UuidLe,
}

impl SendItemLe {
    pub fn to_proto(&self, ctx: &Context) -> SendItem {
        SendItem::with_serialize_value(ChannelCookie(self.cookie.get(ctx)), &()).unwrap()
    }
}

impl UpdateContext for SendItem {
    fn update_context(&self, ctx: &mut Context) {
        ctx.add_uuid(self.cookie.0);
    }
}

#[derive(Debug, Arbitrary)]
pub struct ItemReceivedLe {
    pub cookie: UuidLe,
}

impl ItemReceivedLe {
    pub fn to_proto(&self, ctx: &Context) -> ItemReceived {
        ItemReceived::with_serialize_value(ChannelCookie(self.cookie.get(ctx)), &()).unwrap()
    }
}

impl UpdateContext for ItemReceived {
    fn update_context(&self, ctx: &mut Context) {
        ctx.add_uuid(self.cookie.0);
    }
}

#[derive(Debug, Arbitrary)]
pub struct AddChannelCapacityLe {
    pub cookie: UuidLe,
    pub capacity: u32,
}

impl AddChannelCapacityLe {
    pub fn to_proto(&self, ctx: &Context) -> AddChannelCapacity {
        AddChannelCapacity {
            cookie: ChannelCookie(self.cookie.get(ctx)),
            capacity: self.capacity,
        }
    }
}

impl UpdateContext for AddChannelCapacity {
    fn update_context(&self, ctx: &mut Context) {
        ctx.add_uuid(self.cookie.0);
    }
}

#[derive(Debug, Arbitrary)]
pub struct SyncLe {
    pub serial: SerialLe,
}

impl SyncLe {
    pub fn to_proto(&self, ctx: &Context) -> Sync {
        Sync {
            serial: self.serial.get(ctx),
        }
    }
}

impl UpdateContext for Sync {
    fn update_context(&self, ctx: &mut Context) {
        ctx.add_serial(self.serial);
    }
}

#[derive(Debug, Arbitrary)]
pub struct SyncReplyLe {
    pub serial: SerialLe,
}

impl SyncReplyLe {
    pub fn to_proto(&self, ctx: &Context) -> SyncReply {
        SyncReply {
            serial: self.serial.get(ctx),
        }
    }
}

impl UpdateContext for SyncReply {
    fn update_context(&self, ctx: &mut Context) {
        ctx.add_serial(self.serial);
    }
}
