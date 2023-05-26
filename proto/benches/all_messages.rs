use aldrin_proto::message::{
    AddChannelCapacity, CallFunction, CallFunctionReply, ChannelEnd, ChannelEndClaimed,
    ChannelEndClosed, ChannelEndWithCapacity, ClaimChannelEnd, ClaimChannelEndReply,
    ClaimChannelEndResult, CloseChannelEnd, CloseChannelEndReply, CloseChannelEndResult, Connect,
    ConnectReply, CreateChannel, CreateChannelReply, CreateObject, CreateObjectReply,
    CreateObjectResult, CreateService, CreateServiceReply, CreateServiceResult, DestroyObject,
    DestroyObjectReply, DestroyObjectResult, DestroyService, DestroyServiceReply,
    DestroyServiceResult, EmitEvent, ItemReceived, Message, MessageOps, ObjectCreatedEvent,
    ObjectDestroyedEvent, Packetizer, QueryObject, QueryObjectReply, QueryObjectResult,
    QueryServiceVersion, QueryServiceVersionReply, QueryServiceVersionResult, SendItem,
    ServiceCreatedEvent, ServiceDestroyedEvent, Shutdown, SubscribeEvent, SubscribeEventReply,
    SubscribeEventResult, SubscribeObjects, SubscribeObjectsReply, SubscribeServices,
    SubscribeServicesReply, Sync, SyncReply, UnsubscribeEvent, UnsubscribeObjects,
    UnsubscribeServices,
};
use aldrin_proto::{
    ChannelCookie, ObjectCookie, ObjectId, ObjectUuid, ServiceCookie, ServiceId, ServiceUuid,
};
use criterion::{black_box, BatchSize, Criterion};
use uuid::Uuid;

pub fn serialize(c: &mut Criterion) {
    c.bench_function("all-messages/serialize", |b| {
        b.iter_batched(
            AllMessages::new,
            |msgs| msgs.serialize(),
            BatchSize::SmallInput,
        )
    });
}

pub fn deserialize(c: &mut Criterion) {
    c.bench_function("all-messages/deserialize", |b| {
        let msgs = AllMessages::new();
        let buf = msgs.serialize();
        b.iter(|| AllMessages::deserialize(black_box(&buf)))
    });
}

struct AllMessages {
    msgs: Vec<Message>,
}

impl AllMessages {
    fn new() -> Self {
        const UUID1: Uuid = uuid::uuid!("00000000-0000-0000-0000-000000000001");
        const UUID2: Uuid = uuid::uuid!("00000000-0000-0000-0000-000000000002");
        const UUID3: Uuid = uuid::uuid!("00000000-0000-0000-0000-000000000003");
        const UUID4: Uuid = uuid::uuid!("00000000-0000-0000-0000-000000000004");

        let mut msgs = Vec::with_capacity(100);

        msgs.push(Connect::with_serialize_value(0, &()).unwrap().into());
        msgs.push(ConnectReply::ok_with_serialize_value(&()).unwrap().into());
        msgs.push(Shutdown.into());

        msgs.push(
            CreateObject {
                serial: 0,
                uuid: ObjectUuid(UUID1),
            }
            .into(),
        );

        msgs.push(
            CreateObjectReply {
                serial: 0,
                result: CreateObjectResult::Ok(ObjectCookie(UUID1)),
            }
            .into(),
        );

        msgs.push(
            DestroyObject {
                serial: 0,
                cookie: ObjectCookie(UUID1),
            }
            .into(),
        );

        msgs.push(
            DestroyObjectReply {
                serial: 0,
                result: DestroyObjectResult::Ok,
            }
            .into(),
        );

        msgs.push(SubscribeObjects { serial: Some(0) }.into());
        msgs.push(SubscribeObjectsReply { serial: 0 }.into());
        msgs.push(UnsubscribeObjects.into());

        msgs.push(
            ObjectCreatedEvent {
                id: ObjectId {
                    uuid: ObjectUuid(UUID1),
                    cookie: ObjectCookie(UUID2),
                },
                serial: Some(0),
            }
            .into(),
        );

        msgs.push(
            ObjectDestroyedEvent {
                id: ObjectId {
                    uuid: ObjectUuid(UUID1),
                    cookie: ObjectCookie(UUID2),
                },
            }
            .into(),
        );

        msgs.push(
            CreateService {
                serial: 0,
                object_cookie: ObjectCookie(UUID1),
                uuid: ServiceUuid(UUID2),
                version: 1,
            }
            .into(),
        );

        msgs.push(
            CreateServiceReply {
                serial: 0,
                result: CreateServiceResult::Ok(ServiceCookie(UUID1)),
            }
            .into(),
        );

        msgs.push(
            DestroyService {
                serial: 0,
                cookie: ServiceCookie(UUID1),
            }
            .into(),
        );

        msgs.push(
            DestroyServiceReply {
                serial: 0,
                result: DestroyServiceResult::Ok,
            }
            .into(),
        );

        msgs.push(SubscribeServices { serial: Some(0) }.into());
        msgs.push(SubscribeServicesReply { serial: 0 }.into());
        msgs.push(UnsubscribeServices.into());

        msgs.push(
            ServiceCreatedEvent {
                id: ServiceId {
                    object_id: ObjectId {
                        uuid: ObjectUuid(UUID1),
                        cookie: ObjectCookie(UUID2),
                    },
                    uuid: ServiceUuid(UUID3),
                    cookie: ServiceCookie(UUID4),
                },
                serial: Some(0),
            }
            .into(),
        );

        msgs.push(
            ServiceDestroyedEvent {
                id: ServiceId {
                    object_id: ObjectId {
                        uuid: ObjectUuid(UUID1),
                        cookie: ObjectCookie(UUID2),
                    },
                    uuid: ServiceUuid(UUID3),
                    cookie: ServiceCookie(UUID4),
                },
            }
            .into(),
        );

        msgs.push(
            CallFunction::with_serialize_value(0, ServiceCookie(UUID1), 1, &())
                .unwrap()
                .into(),
        );

        msgs.push(
            CallFunctionReply::ok_with_serialize_value(0, &())
                .unwrap()
                .into(),
        );

        msgs.push(
            SubscribeEvent {
                serial: Some(0),
                service_cookie: ServiceCookie(UUID1),
                event: 1,
            }
            .into(),
        );

        msgs.push(
            SubscribeEventReply {
                serial: 0,
                result: SubscribeEventResult::Ok,
            }
            .into(),
        );

        msgs.push(
            UnsubscribeEvent {
                service_cookie: ServiceCookie(UUID1),
                event: 0,
            }
            .into(),
        );

        msgs.push(
            EmitEvent::with_serialize_value(ServiceCookie(UUID1), 0, &())
                .unwrap()
                .into(),
        );

        msgs.push(
            QueryObject {
                serial: 0,
                uuid: ObjectUuid(UUID1),
                with_services: false,
            }
            .into(),
        );

        msgs.push(
            QueryObjectReply {
                serial: 0,
                result: QueryObjectResult::Service {
                    uuid: ServiceUuid(UUID1),
                    cookie: ServiceCookie(UUID2),
                },
            }
            .into(),
        );

        msgs.push(
            QueryServiceVersion {
                serial: 0,
                cookie: ServiceCookie(UUID1),
            }
            .into(),
        );

        msgs.push(
            QueryServiceVersionReply {
                serial: 0,
                result: QueryServiceVersionResult::Ok(1),
            }
            .into(),
        );

        msgs.push(
            CreateChannel {
                serial: 0,
                end: ChannelEndWithCapacity::Receiver(1),
            }
            .into(),
        );

        msgs.push(
            CreateChannelReply {
                serial: 0,
                cookie: ChannelCookie(UUID1),
            }
            .into(),
        );

        msgs.push(
            CloseChannelEnd {
                serial: 0,
                cookie: ChannelCookie(UUID1),
                end: ChannelEnd::Sender,
            }
            .into(),
        );

        msgs.push(
            CloseChannelEndReply {
                serial: 0,
                result: CloseChannelEndResult::Ok,
            }
            .into(),
        );

        msgs.push(
            ChannelEndClosed {
                cookie: ChannelCookie(UUID1),
                end: ChannelEnd::Sender,
            }
            .into(),
        );

        msgs.push(
            ClaimChannelEnd {
                serial: 0,
                cookie: ChannelCookie(UUID1),
                end: ChannelEndWithCapacity::Receiver(1),
            }
            .into(),
        );

        msgs.push(
            ClaimChannelEndReply {
                serial: 0,
                result: ClaimChannelEndResult::SenderClaimed(1),
            }
            .into(),
        );

        msgs.push(
            ChannelEndClaimed {
                cookie: ChannelCookie(UUID1),
                end: ChannelEndWithCapacity::Receiver(0),
            }
            .into(),
        );

        msgs.push(
            SendItem::with_serialize_value(ChannelCookie(UUID1), &())
                .unwrap()
                .into(),
        );

        msgs.push(
            ItemReceived::with_serialize_value(ChannelCookie(UUID1), &())
                .unwrap()
                .into(),
        );

        msgs.push(
            AddChannelCapacity {
                cookie: ChannelCookie(UUID1),
                capacity: 0,
            }
            .into(),
        );

        msgs.push(Sync { serial: 0 }.into());
        msgs.push(SyncReply { serial: 0 }.into());

        Self { msgs }
    }

    fn serialize(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(64 * 1024);

        for msg in self.msgs {
            buf.extend_from_slice(&msg.serialize_message().unwrap());
        }

        buf
    }

    fn deserialize(buf: &[u8]) -> Self {
        let mut packetizer = Packetizer::new();
        packetizer.extend_from_slice(buf);

        let mut msgs = Vec::with_capacity(100);
        while let Some(msg) = packetizer.next_message() {
            let msg = Message::deserialize_message(msg).unwrap();
            msgs.push(msg);
        }

        Self { msgs }
    }
}
