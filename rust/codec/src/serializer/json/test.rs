use super::Json;
use crate::Serializer;
use aldrin_proto::*;
use maplit::{hashmap, hashset};
use serde_json::json;
use std::str::from_utf8;
use uuid::{uuid, Uuid};

const UUID1: Uuid = uuid!("2a6c8683-48b9-4c82-9561-44201d707232");
const UUID2: Uuid = uuid!("1d0720e6-6944-4801-a07f-aaa075a46339");
const UUID3: Uuid = uuid!("8892f6c4-f4c8-45c7-9373-0b40a6bb3097");
const UUID4: Uuid = uuid!("7ff46206-eddc-4a9e-8ce9-707b1f35a1ba");

fn test_message(m: Message, j: serde_json::Value) {
    let mut ser = Json::with_pretty(false);

    let buf = ser.serialize(m.clone()).unwrap();
    let j2_str = from_utf8(&buf).unwrap();
    let j2: serde_json::Value = serde_json::from_str(j2_str).unwrap();
    assert_eq!(j, j2);

    let m2: Message = serde_json::from_value(j).unwrap();
    assert_eq!(m, m2);
}

fn test_value(v: Value, j: serde_json::Value) {
    let mv = Message::CallFunctionReply(CallFunctionReply {
        serial: 0,
        result: CallFunctionResult::Ok(v),
    });
    let mj = json!({"call-function-reply": {"serial": 0, "result": {"ok": j}}});
    test_message(mv, mj);
}

#[test]
fn value_none() {
    test_value(Value::None, json!("none"));
}

#[test]
fn value_bool() {
    test_value(Value::Bool(true), json!({"bool": true}));
    test_value(Value::Bool(false), json!({"bool": false}));
}

#[test]
fn value_u8() {
    test_value(Value::U8(u8::MIN), json!({ "u8": u8::MIN }));
    test_value(Value::U8(u8::MAX), json!({ "u8": u8::MAX }));
}

#[test]
fn value_i8() {
    test_value(Value::I8(i8::MIN), json!({ "i8": i8::MIN }));
    test_value(Value::I8(i8::MAX), json!({ "i8": i8::MAX }));
}

#[test]
fn value_u16() {
    test_value(Value::U16(u16::MIN), json!({ "u16": u16::MIN }));
    test_value(Value::U16(u16::MAX), json!({ "u16": u16::MAX }));
}

#[test]
fn value_i16() {
    test_value(Value::I16(i16::MIN), json!({ "i16": i16::MIN }));
    test_value(Value::I16(i16::MAX), json!({ "i16": i16::MAX }));
}

#[test]
fn value_u32() {
    test_value(Value::U32(u32::MIN), json!({ "u32": u32::MIN }));
    test_value(Value::U32(u32::MAX), json!({ "u32": u32::MAX }));
}

#[test]
fn value_i32() {
    test_value(Value::I32(i32::MIN), json!({ "i32": i32::MIN }));
    test_value(Value::I32(i32::MAX), json!({ "i32": i32::MAX }));
}

#[test]
fn value_u64() {
    test_value(Value::U64(u64::MIN), json!({ "u64": u64::MIN }));
    test_value(Value::U64(u64::MAX), json!({ "u64": u64::MAX }));
}

#[test]
fn value_i64() {
    test_value(Value::I64(i64::MIN), json!({ "i64": i64::MIN }));
    test_value(Value::I64(i64::MAX), json!({ "i64": i64::MAX }));
}

#[test]
fn value_f32() {
    test_value(Value::F32(0.0), json!({"f32": 0.0}));
}

#[test]
fn value_f64() {
    test_value(Value::F64(0.0), json!({"f64": 0.0}));
}

#[test]
fn value_string() {
    test_value(Value::String("".to_owned()), json!({"string": ""}));
    test_value(Value::String("foo".to_owned()), json!({"string": "foo"}));
    test_value(
        Value::String("f\0\0".to_owned()),
        json!({"string": "f\0\0"}),
    );
}

#[test]
fn value_uuid() {
    test_value(Value::Uuid(Uuid::nil()), json!({ "uuid": Uuid::nil() }));
    test_value(Value::Uuid(UUID1), json!({ "uuid": UUID1 }));
}

#[test]
fn value_object_id() {
    test_value(
        Value::ObjectId(ObjectId {
            uuid: ObjectUuid(UUID1),
            cookie: ObjectCookie(UUID2),
        }),
        json!({"object-id": {"uuid": UUID1, "cookie": UUID2}}),
    );
}

#[test]
fn value_service_id() {
    test_value(
        Value::ServiceId(ServiceId {
            object_id: ObjectId {
                uuid: ObjectUuid(UUID1),
                cookie: ObjectCookie(UUID2),
            },
            uuid: ServiceUuid(UUID3),
            cookie: ServiceCookie(UUID4),
        }),
        json!({"service-id": {
            "object-id": {
                "uuid": UUID1,
                "cookie": UUID2,
            },
            "uuid": UUID3,
            "cookie": UUID4,
        }}),
    );
}

#[test]
fn value_vec() {
    test_value(Value::Vec(vec![]), json!({"vec": []}));
    test_value(Value::Vec(vec![Value::None]), json!({"vec": ["none"]}));
    test_value(
        Value::Vec(vec![Value::None, Value::U8(0)]),
        json!({"vec": ["none", {"u8": 0}]}),
    );
}

#[test]
fn value_bytes() {
    test_value(Value::Bytes(vec![]), json!({"bytes": []}));
    test_value(Value::Bytes(vec![0]), json!({"bytes": [0]}));
    test_value(Value::Bytes(vec![0, 1]), json!({"bytes": [0, 1]}));
}

#[test]
fn value_u8_map() {
    test_value(Value::U8Map(hashmap![]), json!({"u8-map": {}}));
    test_value(
        Value::U8Map(hashmap![u8::MIN => Value::None, u8::MAX => Value::U8(0)]),
        json!({"u8-map": {u8::MIN.to_string(): "none", u8::MAX.to_string(): {"u8": 0}}}),
    );
}

#[test]
fn value_i8_map() {
    test_value(Value::I8Map(hashmap![]), json!({"i8-map": {}}));
    test_value(
        Value::I8Map(hashmap![i8::MIN => Value::None, i8::MAX => Value::U8(0)]),
        json!({"i8-map": {i8::MIN.to_string(): "none", i8::MAX.to_string(): {"u8": 0}}}),
    );
}

#[test]
fn value_u16_map() {
    test_value(Value::U16Map(hashmap![]), json!({"u16-map": {}}));
    test_value(
        Value::U16Map(hashmap![u16::MIN => Value::None, u16::MAX => Value::U8(0)]),
        json!({"u16-map": {u16::MIN.to_string(): "none", u16::MAX.to_string(): {"u8": 0}}}),
    );
}

#[test]
fn value_i16_map() {
    test_value(Value::I16Map(hashmap![]), json!({"i16-map": {}}));
    test_value(
        Value::I16Map(hashmap![i16::MIN => Value::None, i16::MAX => Value::U8(0)]),
        json!({"i16-map": {i16::MIN.to_string(): "none", i16::MAX.to_string(): {"u8": 0}}}),
    );
}

#[test]
fn value_u32_map() {
    test_value(Value::U32Map(hashmap![]), json!({"u32-map": {}}));
    test_value(
        Value::U32Map(hashmap![u32::MIN => Value::None, u32::MAX => Value::U8(0)]),
        json!({"u32-map": {u32::MIN.to_string(): "none", u32::MAX.to_string(): {"u8": 0}}}),
    );
}

#[test]
fn value_i32_map() {
    test_value(Value::I32Map(hashmap![]), json!({"i32-map": {}}));
    test_value(
        Value::I32Map(hashmap![i32::MIN => Value::None, i32::MAX => Value::U8(0)]),
        json!({"i32-map": {i32::MIN.to_string(): "none", i32::MAX.to_string(): {"u8": 0}}}),
    );
}

#[test]
fn value_u64_map() {
    test_value(Value::U64Map(hashmap![]), json!({"u64-map": {}}));
    test_value(
        Value::U64Map(hashmap![u64::MIN => Value::None, u64::MAX => Value::U8(0)]),
        json!({"u64-map": {u64::MIN.to_string(): "none", u64::MAX.to_string(): {"u8": 0}}}),
    );
}

#[test]
fn value_i64_map() {
    test_value(Value::I64Map(hashmap![]), json!({"i64-map": {}}));
    test_value(
        Value::I64Map(hashmap![i64::MIN => Value::None, i64::MAX => Value::U8(0)]),
        json!({"i64-map": {i64::MIN.to_string(): "none", i64::MAX.to_string(): {"u8": 0}}}),
    );
}

#[test]
fn value_string_map() {
    test_value(Value::StringMap(hashmap![]), json!({"string-map": {}}));
    test_value(
        Value::StringMap(hashmap!["".to_owned() => Value::None]),
        json!({"string-map": {"": "none"}}),
    );
    test_value(
        Value::StringMap(hashmap!["f\0\0".to_owned() => Value::None]),
        json!({"string-map": {"f\0\0": "none"}}),
    );
}

#[test]
fn value_uuid_map() {
    test_value(Value::UuidMap(hashmap![]), json!({"uuid-map": {}}));
    test_value(
        Value::UuidMap(hashmap![UUID1 => Value::None]),
        json!({"uuid-map": {UUID1.to_string(): "none"}}),
    );
}

#[test]
fn value_u8_set() {
    test_value(Value::U8Set(hashset![]), json!({"u8-set": []}));
    test_value(Value::U8Set(hashset![0]), json!({"u8-set": [0]}));
}

#[test]
fn value_i8_set() {
    test_value(Value::I8Set(hashset![]), json!({"i8-set": []}));
    test_value(Value::I8Set(hashset![0]), json!({"i8-set": [0]}));
}

#[test]
fn value_u16_set() {
    test_value(Value::U16Set(hashset![]), json!({"u16-set": []}));
    test_value(Value::U16Set(hashset![0]), json!({"u16-set": [0]}));
}

#[test]
fn value_i16_set() {
    test_value(Value::I16Set(hashset![]), json!({"i16-set": []}));
    test_value(Value::I16Set(hashset![0]), json!({"i16-set": [0]}));
}

#[test]
fn value_u32_set() {
    test_value(Value::U32Set(hashset![]), json!({"u32-set": []}));
    test_value(Value::U32Set(hashset![0]), json!({"u32-set": [0]}));
}

#[test]
fn value_i32_set() {
    test_value(Value::I32Set(hashset![]), json!({"i32-set": []}));
    test_value(Value::I32Set(hashset![0]), json!({"i32-set": [0]}));
}

#[test]
fn value_u64_set() {
    test_value(Value::U64Set(hashset![]), json!({"u64-set": []}));
    test_value(Value::U64Set(hashset![0]), json!({"u64-set": [0]}));
}

#[test]
fn value_i64_set() {
    test_value(Value::I64Set(hashset![]), json!({"i64-set": []}));
    test_value(Value::I64Set(hashset![0]), json!({"i64-set": [0]}));
}

#[test]
fn value_string_set() {
    test_value(Value::StringSet(hashset![]), json!({"string-set": []}));
    test_value(
        Value::StringSet(hashset!["".to_owned()]),
        json!({"string-set": [""]}),
    );
    test_value(
        Value::StringSet(hashset!["foo".to_owned()]),
        json!({"string-set": ["foo"]}),
    );
}

#[test]
fn value_uuid_set() {
    test_value(Value::UuidSet(hashset![]), json!({"uuid-set": []}));
    test_value(
        Value::UuidSet(hashset![UUID1]),
        json!({ "uuid-set": [UUID1] }),
    );
}

#[test]
fn value_struct() {
    test_value(Value::Struct(hashmap![]), json!({"struct": {}}));
    test_value(
        Value::Struct(hashmap![0 => Value::None]),
        json!({"struct": {"0": "none"}}),
    );
}

#[test]
fn value_enum() {
    test_value(
        Value::Enum {
            variant: 0,
            value: Box::new(Value::None),
        },
        json!({"enum": {"variant": 0, "value": "none"}}),
    );
    test_value(
        Value::Enum {
            variant: 0,
            value: Box::new(Value::U8(0)),
        },
        json!({"enum": {"variant": 0, "value": {"u8": 0}}}),
    );
}

#[test]
fn message_connect() {
    test_message(
        Message::Connect(Connect { version: 0 }),
        json!({"connect": {"version": 0}}),
    );
}

#[test]
fn message_connect_reply() {
    test_message(
        Message::ConnectReply(ConnectReply::Ok),
        json!({"connect-reply": "ok"}),
    );
    test_message(
        Message::ConnectReply(ConnectReply::VersionMismatch(0)),
        json!({"connect-reply": {"version-mismatch": 0}}),
    );
}

#[test]
fn message_shutdown() {
    test_message(Message::Shutdown(()), json!({ "shutdown": null }));
}

#[test]
fn message_create_object() {
    test_message(
        Message::CreateObject(CreateObject {
            serial: 0,
            uuid: UUID1,
        }),
        json!({"create-object": {"serial": 0, "uuid": UUID1}}),
    );
}

#[test]
fn message_create_object_reply() {
    test_message(
        Message::CreateObjectReply(CreateObjectReply {
            serial: 0,
            result: CreateObjectResult::Ok(UUID1),
        }),
        json!({"create-object-reply": {"serial": 0, "result": {"ok": UUID1}}}),
    );
    test_message(
        Message::CreateObjectReply(CreateObjectReply {
            serial: 0,
            result: CreateObjectResult::DuplicateObject,
        }),
        json!({"create-object-reply": {"serial": 0, "result": "duplicate-object"}}),
    );
}

#[test]
fn message_destroy_object() {
    test_message(
        Message::DestroyObject(DestroyObject {
            serial: 0,
            cookie: UUID1,
        }),
        json!({"destroy-object": {"serial": 0, "cookie": UUID1}}),
    );
}

#[test]
fn message_destroy_object_reply() {
    test_message(
        Message::DestroyObjectReply(DestroyObjectReply {
            serial: 0,
            result: DestroyObjectResult::Ok,
        }),
        json!({"destroy-object-reply": {"serial": 0, "result": "ok"}}),
    );
    test_message(
        Message::DestroyObjectReply(DestroyObjectReply {
            serial: 0,
            result: DestroyObjectResult::InvalidObject,
        }),
        json!({"destroy-object-reply": {"serial": 0, "result": "invalid-object"}}),
    );
    test_message(
        Message::DestroyObjectReply(DestroyObjectReply {
            serial: 0,
            result: DestroyObjectResult::ForeignObject,
        }),
        json!({"destroy-object-reply": {"serial": 0, "result": "foreign-object"}}),
    );
}

#[test]
fn message_subscribe_objects() {
    test_message(
        Message::SubscribeObjects(SubscribeObjects { serial: None }),
        json!({"subscribe-objects": {"serial": null}}),
    );
    test_message(
        Message::SubscribeObjects(SubscribeObjects { serial: Some(0) }),
        json!({"subscribe-objects": {"serial": 0}}),
    );
}

#[test]
fn message_subscribe_objects_reply() {
    test_message(
        Message::SubscribeObjectsReply(SubscribeObjectsReply { serial: 0 }),
        json!({"subscribe-objects-reply": {"serial": 0}}),
    );
}

#[test]
fn message_unsubscribe_objects() {
    test_message(
        Message::UnsubscribeObjects(()),
        json!({ "unsubscribe-objects": null }),
    );
}

#[test]
fn message_object_created_event() {
    test_message(
        Message::ObjectCreatedEvent(ObjectCreatedEvent {
            uuid: UUID1,
            cookie: UUID2,
            serial: None,
        }),
        json!({"object-created-event": {"uuid": UUID1, "cookie": UUID2, "serial": null}}),
    );
    test_message(
        Message::ObjectCreatedEvent(ObjectCreatedEvent {
            uuid: UUID1,
            cookie: UUID2,
            serial: Some(0),
        }),
        json!({"object-created-event": {"uuid": UUID1, "cookie": UUID2, "serial": 0}}),
    );
}

#[test]
fn message_object_destroyed_event() {
    test_message(
        Message::ObjectDestroyedEvent(ObjectDestroyedEvent {
            uuid: UUID1,
            cookie: UUID2,
        }),
        json!({"object-destroyed-event": {"uuid": UUID1, "cookie": UUID2}}),
    );
}

#[test]
fn message_create_service() {
    test_message(
        Message::CreateService(CreateService {
            serial: 0,
            object_cookie: UUID1,
            uuid: UUID2,
            version: 1,
        }),
        json!({"create-service": {
            "serial": 0,
            "object-cookie": UUID1,
            "uuid": UUID2,
            "version": 1,
        }}),
    );
}

#[test]
fn message_create_service_reply() {
    test_message(
        Message::CreateServiceReply(CreateServiceReply {
            serial: 0,
            result: CreateServiceResult::Ok(UUID1),
        }),
        json!({"create-service-reply": {"serial": 0, "result": {"ok": UUID1}}}),
    );
    test_message(
        Message::CreateServiceReply(CreateServiceReply {
            serial: 0,
            result: CreateServiceResult::DuplicateService,
        }),
        json!({"create-service-reply": {"serial": 0, "result": "duplicate-service"}}),
    );
    test_message(
        Message::CreateServiceReply(CreateServiceReply {
            serial: 0,
            result: CreateServiceResult::InvalidObject,
        }),
        json!({"create-service-reply": {"serial": 0, "result": "invalid-object"}}),
    );
    test_message(
        Message::CreateServiceReply(CreateServiceReply {
            serial: 0,
            result: CreateServiceResult::ForeignObject,
        }),
        json!({"create-service-reply": {"serial": 0, "result": "foreign-object"}}),
    );
}

#[test]
fn message_destroy_service() {
    test_message(
        Message::DestroyService(DestroyService {
            serial: 0,
            cookie: UUID1,
        }),
        json!({"destroy-service": {"serial": 0, "cookie": UUID1}}),
    );
}

#[test]
fn message_destroy_service_reply() {
    test_message(
        Message::DestroyServiceReply(DestroyServiceReply {
            serial: 0,
            result: DestroyServiceResult::Ok,
        }),
        json!({"destroy-service-reply": {"serial": 0, "result": "ok"}}),
    );
    test_message(
        Message::DestroyServiceReply(DestroyServiceReply {
            serial: 0,
            result: DestroyServiceResult::InvalidService,
        }),
        json!({"destroy-service-reply": {"serial": 0, "result": "invalid-service"}}),
    );
    test_message(
        Message::DestroyServiceReply(DestroyServiceReply {
            serial: 0,
            result: DestroyServiceResult::ForeignObject,
        }),
        json!({"destroy-service-reply": {"serial": 0, "result": "foreign-object"}}),
    );
}

#[test]
fn message_subscribe_services() {
    test_message(
        Message::SubscribeServices(SubscribeServices { serial: None }),
        json!({"subscribe-services": {"serial": null}}),
    );
    test_message(
        Message::SubscribeServices(SubscribeServices { serial: Some(0) }),
        json!({"subscribe-services": {"serial": 0}}),
    );
}

#[test]
fn message_subscribe_services_reply() {
    test_message(
        Message::SubscribeServicesReply(SubscribeServicesReply { serial: 0 }),
        json!({"subscribe-services-reply": {"serial": 0}}),
    );
}

#[test]
fn message_unsubscribe_services() {
    test_message(
        Message::UnsubscribeServices(()),
        json!({ "unsubscribe-services": null }),
    );
}

#[test]
fn message_service_created_event() {
    test_message(
        Message::ServiceCreatedEvent(ServiceCreatedEvent {
            object_uuid: UUID1,
            object_cookie: UUID2,
            uuid: UUID3,
            cookie: UUID4,
            serial: None,
        }),
        json!({"service-created-event": {
            "object-uuid": UUID1,
            "object-cookie": UUID2,
            "uuid": UUID3,
            "cookie": UUID4,
            "serial": null,
        }}),
    );
    test_message(
        Message::ServiceCreatedEvent(ServiceCreatedEvent {
            object_uuid: UUID1,
            object_cookie: UUID2,
            uuid: UUID3,
            cookie: UUID4,
            serial: Some(0),
        }),
        json!({"service-created-event": {
            "object-uuid": UUID1,
            "object-cookie": UUID2,
            "uuid": UUID3,
            "cookie": UUID4,
            "serial": 0,
        }}),
    );
}

#[test]
fn message_service_destroyed_event() {
    test_message(
        Message::ServiceDestroyedEvent(ServiceDestroyedEvent {
            object_uuid: UUID1,
            object_cookie: UUID2,
            uuid: UUID3,
            cookie: UUID4,
        }),
        json!({"service-destroyed-event": {
            "object-uuid": UUID1,
            "object-cookie": UUID2,
            "uuid": UUID3,
            "cookie": UUID4,
        }}),
    );
}

#[test]
fn message_call_function() {
    test_message(
        Message::CallFunction(CallFunction {
            serial: 0,
            service_cookie: UUID1,
            function: 1,
            args: Value::None,
        }),
        json!({"call-function": {
            "serial": 0,
            "service-cookie": UUID1,
            "function": 1,
            "args": "none",
        }}),
    );
}

#[test]
fn message_call_function_reply() {
    test_message(
        Message::CallFunctionReply(CallFunctionReply {
            serial: 0,
            result: CallFunctionResult::Ok(Value::None),
        }),
        json!({"call-function-reply": {"serial": 0, "result": {"ok": "none"}}}),
    );
    test_message(
        Message::CallFunctionReply(CallFunctionReply {
            serial: 0,
            result: CallFunctionResult::Err(Value::None),
        }),
        json!({"call-function-reply": {"serial": 0, "result": {"err": "none"}}}),
    );
    test_message(
        Message::CallFunctionReply(CallFunctionReply {
            serial: 0,
            result: CallFunctionResult::Aborted,
        }),
        json!({"call-function-reply": {"serial": 0, "result": "aborted"}}),
    );
    test_message(
        Message::CallFunctionReply(CallFunctionReply {
            serial: 0,
            result: CallFunctionResult::InvalidService,
        }),
        json!({"call-function-reply": {"serial": 0, "result": "invalid-service"}}),
    );
    test_message(
        Message::CallFunctionReply(CallFunctionReply {
            serial: 0,
            result: CallFunctionResult::InvalidFunction,
        }),
        json!({"call-function-reply": {"serial": 0, "result": "invalid-function"}}),
    );
    test_message(
        Message::CallFunctionReply(CallFunctionReply {
            serial: 0,
            result: CallFunctionResult::InvalidArgs,
        }),
        json!({"call-function-reply": {"serial": 0, "result": "invalid-args"}}),
    );
}

#[test]
fn message_subscribe_event() {
    test_message(
        Message::SubscribeEvent(SubscribeEvent {
            serial: None,
            service_cookie: UUID1,
            event: 0,
        }),
        json!({"subscribe-event": {"serial": null, "service-cookie": UUID1, "event": 0}}),
    );
    test_message(
        Message::SubscribeEvent(SubscribeEvent {
            serial: Some(0),
            service_cookie: UUID1,
            event: 1,
        }),
        json!({"subscribe-event": {"serial": 0, "service-cookie": UUID1, "event": 1}}),
    );
}

#[test]
fn message_subscribe_event_reply() {
    test_message(
        Message::SubscribeEventReply(SubscribeEventReply {
            serial: 0,
            result: SubscribeEventResult::Ok,
        }),
        json!({"subscribe-event-reply": {"serial": 0, "result": "ok"}}),
    );
    test_message(
        Message::SubscribeEventReply(SubscribeEventReply {
            serial: 0,
            result: SubscribeEventResult::InvalidService,
        }),
        json!({"subscribe-event-reply": {"serial": 0, "result": "invalid-service"}}),
    );
}

#[test]
fn message_unsubscribe_event() {
    test_message(
        Message::UnsubscribeEvent(UnsubscribeEvent {
            service_cookie: UUID1,
            event: 0,
        }),
        json!({"unsubscribe-event": {"service-cookie": UUID1, "event": 0}}),
    );
}

#[test]
fn message_emit_event() {
    test_message(
        Message::EmitEvent(EmitEvent {
            service_cookie: UUID1,
            event: 0,
            args: Value::None,
        }),
        json!({"emit-event": {"service-cookie": UUID1, "event": 0, "args": "none"}}),
    );
}

#[test]
fn message_query_object() {
    test_message(
        Message::QueryObject(QueryObject {
            serial: 0,
            uuid: UUID1,
            with_services: true,
        }),
        json!({"query-object": {"serial": 0, "uuid": UUID1, "with-services": true}}),
    );
}

#[test]
fn message_query_object_reply() {
    test_message(
        Message::QueryObjectReply(QueryObjectReply {
            serial: 0,
            result: QueryObjectResult::Cookie(UUID1),
        }),
        json!({"query-object-reply": {"serial": 0, "result": {"cookie": UUID1}}}),
    );
    test_message(
        Message::QueryObjectReply(QueryObjectReply {
            serial: 0,
            result: QueryObjectResult::Service {
                uuid: UUID1,
                cookie: UUID2,
            },
        }),
        json!({"query-object-reply": {
            "serial": 0,
            "result": {
                "service": {
                    "uuid": UUID1,
                    "cookie": UUID2,
                }
            }
        }}),
    );
    test_message(
        Message::QueryObjectReply(QueryObjectReply {
            serial: 0,
            result: QueryObjectResult::Done,
        }),
        json!({"query-object-reply": {"serial": 0, "result": "done"}}),
    );
    test_message(
        Message::QueryObjectReply(QueryObjectReply {
            serial: 0,
            result: QueryObjectResult::InvalidObject,
        }),
        json!({"query-object-reply": {"serial": 0, "result": "invalid-object"}}),
    );
}

#[test]
fn message_query_service_version() {
    test_message(
        Message::QueryServiceVersion(QueryServiceVersion {
            serial: 0,
            cookie: UUID1,
        }),
        json!({"query-service-version": {"serial": 0, "cookie": UUID1}}),
    );
}

#[test]
fn message_query_service_version_reply() {
    test_message(
        Message::QueryServiceVersionReply(QueryServiceVersionReply {
            serial: 0,
            result: QueryServiceVersionResult::Ok(1),
        }),
        json!({"query-service-version-reply": {"serial": 0, "result": {"ok": 1}}}),
    );
    test_message(
        Message::QueryServiceVersionReply(QueryServiceVersionReply {
            serial: 0,
            result: QueryServiceVersionResult::InvalidService,
        }),
        json!({"query-service-version-reply": {"serial": 0, "result": "invalid-service"}}),
    );
}
