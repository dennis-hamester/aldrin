use super::{JsonSerializer, Serializer};
use aldrin_proto::*;
use bytes::BytesMut;
use maplit::{hashmap, hashset};
use std::str::from_utf8;
use uuid::Uuid;

fn test_message(m: Message, j: &str) {
    let mut ser = JsonSerializer::new(false);
    let mut buf = BytesMut::new();

    ser.serialize(m.clone(), &mut buf).unwrap();
    let j2 = from_utf8(&buf).unwrap();
    assert_eq!(j, j2);

    let m2 = ser.deserialize(buf.freeze()).unwrap();
    assert_eq!(m, m2);
}

fn test_value(v: Value, j: &str) {
    let mv = Message::CallFunctionReply(CallFunctionReply {
        serial: 0,
        result: CallFunctionResult::Ok(v),
    });
    let mj = format!(
        "{{\"call-function-reply\":{{\"serial\":0,\"result\":{{\"ok\":{}}}}}}}",
        j
    );
    test_message(mv, &mj);
}

#[test]
fn value_none() {
    test_value(Value::None, "\"none\"");
}

#[test]
fn value_bool() {
    test_value(Value::Bool(true), "{\"bool\":true}");
    test_value(Value::Bool(false), "{\"bool\":false}");
}

#[test]
fn value_u8() {
    test_value(Value::U8(u8::MIN), "{\"u8\":0}");
    test_value(Value::U8(u8::MAX), "{\"u8\":255}");
}

#[test]
fn value_i8() {
    test_value(Value::I8(i8::MIN), "{\"i8\":-128}");
    test_value(Value::I8(i8::MAX), "{\"i8\":127}");
}

#[test]
fn value_u16() {
    test_value(Value::U16(u16::MIN), "{\"u16\":0}");
    test_value(Value::U16(u16::MAX), "{\"u16\":65535}");
}

#[test]
fn value_i16() {
    test_value(Value::I16(i16::MIN), "{\"i16\":-32768}");
    test_value(Value::I16(i16::MAX), "{\"i16\":32767}");
}

#[test]
fn value_u32() {
    test_value(Value::U32(u32::MIN), "{\"u32\":0}");
    test_value(Value::U32(u32::MAX), "{\"u32\":4294967295}");
}

#[test]
fn value_i32() {
    test_value(Value::I32(i32::MIN), "{\"i32\":-2147483648}");
    test_value(Value::I32(i32::MAX), "{\"i32\":2147483647}");
}

#[test]
fn value_u64() {
    test_value(Value::U64(u64::MIN), "{\"u64\":0}");
    test_value(Value::U64(u64::MAX), "{\"u64\":18446744073709551615}");
}

#[test]
fn value_i64() {
    test_value(Value::I64(i64::MIN), "{\"i64\":-9223372036854775808}");
    test_value(Value::I64(i64::MAX), "{\"i64\":9223372036854775807}");
}

#[test]
fn value_f32() {
    test_value(Value::F32(0.0), "{\"f32\":0.0}");
}

#[test]
fn value_f64() {
    test_value(Value::F64(0.0), "{\"f64\":0.0}");
}

#[test]
fn value_string() {
    test_value(Value::String("".to_owned()), "{\"string\":\"\"}");
    test_value(Value::String("foo".to_owned()), "{\"string\":\"foo\"}");
    test_value(
        Value::String("f\0\0".to_owned()),
        "{\"string\":\"f\\u0000\\u0000\"}",
    );
}

#[test]
fn value_uuid() {
    test_value(
        Value::Uuid(Uuid::nil()),
        "{\"uuid\":\"00000000-0000-0000-0000-000000000000\"}",
    );
}

#[test]
fn value_vec() {
    test_value(Value::Vec(vec![]), "{\"vec\":[]}");
    test_value(Value::Vec(vec![Value::None]), "{\"vec\":[\"none\"]}");
}

#[test]
fn value_bytes() {
    test_value(Value::Bytes(vec![]), "{\"bytes\":[]}");
    test_value(Value::Bytes(vec![0, 1]), "{\"bytes\":[0,1]}");
}

#[test]
fn value_u8_map() {
    test_value(Value::U8Map(hashmap![]), "{\"u8-map\":{}}");
    test_value(
        Value::U8Map(hashmap![0 => Value::None]),
        "{\"u8-map\":{\"0\":\"none\"}}",
    );
}

#[test]
fn value_i8_map() {
    test_value(Value::I8Map(hashmap![]), "{\"i8-map\":{}}");
    test_value(
        Value::I8Map(hashmap![0 => Value::None]),
        "{\"i8-map\":{\"0\":\"none\"}}",
    );
}

#[test]
fn value_u16_map() {
    test_value(Value::U16Map(hashmap![]), "{\"u16-map\":{}}");
    test_value(
        Value::U16Map(hashmap![0 => Value::None]),
        "{\"u16-map\":{\"0\":\"none\"}}",
    );
}

#[test]
fn value_i16_map() {
    test_value(Value::I16Map(hashmap![]), "{\"i16-map\":{}}");
    test_value(
        Value::I16Map(hashmap![0 => Value::None]),
        "{\"i16-map\":{\"0\":\"none\"}}",
    );
}

#[test]
fn value_u32_map() {
    test_value(Value::U32Map(hashmap![]), "{\"u32-map\":{}}");
    test_value(
        Value::U32Map(hashmap![0 => Value::None]),
        "{\"u32-map\":{\"0\":\"none\"}}",
    );
}

#[test]
fn value_i32_map() {
    test_value(Value::I32Map(hashmap![]), "{\"i32-map\":{}}");
    test_value(
        Value::I32Map(hashmap![0 => Value::None]),
        "{\"i32-map\":{\"0\":\"none\"}}",
    );
}

#[test]
fn value_u64_map() {
    test_value(Value::U64Map(hashmap![]), "{\"u64-map\":{}}");
    test_value(
        Value::U64Map(hashmap![0 => Value::None]),
        "{\"u64-map\":{\"0\":\"none\"}}",
    );
}

#[test]
fn value_i64_map() {
    test_value(Value::I64Map(hashmap![]), "{\"i64-map\":{}}");
    test_value(
        Value::I64Map(hashmap![0 => Value::None]),
        "{\"i64-map\":{\"0\":\"none\"}}",
    );
}

#[test]
fn value_string_map() {
    test_value(Value::StringMap(hashmap![]), "{\"string-map\":{}}");
    test_value(
        Value::StringMap(hashmap!["".to_owned() => Value::None]),
        "{\"string-map\":{\"\":\"none\"}}",
    );
    test_value(
        Value::StringMap(hashmap!["foo".to_owned() => Value::None]),
        "{\"string-map\":{\"foo\":\"none\"}}",
    );
}

#[test]
fn value_uuid_map() {
    test_value(Value::UuidMap(hashmap![]), "{\"uuid-map\":{}}");
    test_value(
        Value::UuidMap(hashmap![Uuid::nil() => Value::None]),
        "{\"uuid-map\":{\"00000000-0000-0000-0000-000000000000\":\"none\"}}",
    );
}

#[test]
fn value_u8_set() {
    test_value(Value::U8Set(hashset![]), "{\"u8-set\":[]}");
    test_value(Value::U8Set(hashset![0]), "{\"u8-set\":[0]}");
}

#[test]
fn value_i8_set() {
    test_value(Value::I8Set(hashset![]), "{\"i8-set\":[]}");
    test_value(Value::I8Set(hashset![0]), "{\"i8-set\":[0]}");
}

#[test]
fn value_u16_set() {
    test_value(Value::U16Set(hashset![]), "{\"u16-set\":[]}");
    test_value(Value::U16Set(hashset![0]), "{\"u16-set\":[0]}");
}

#[test]
fn value_i16_set() {
    test_value(Value::I16Set(hashset![]), "{\"i16-set\":[]}");
    test_value(Value::I16Set(hashset![0]), "{\"i16-set\":[0]}");
}

#[test]
fn value_u32_set() {
    test_value(Value::U32Set(hashset![]), "{\"u32-set\":[]}");
    test_value(Value::U32Set(hashset![0]), "{\"u32-set\":[0]}");
}

#[test]
fn value_i32_set() {
    test_value(Value::I32Set(hashset![]), "{\"i32-set\":[]}");
    test_value(Value::I32Set(hashset![0]), "{\"i32-set\":[0]}");
}

#[test]
fn value_u64_set() {
    test_value(Value::U64Set(hashset![]), "{\"u64-set\":[]}");
    test_value(Value::U64Set(hashset![0]), "{\"u64-set\":[0]}");
}

#[test]
fn value_i64_set() {
    test_value(Value::I64Set(hashset![]), "{\"i64-set\":[]}");
    test_value(Value::I64Set(hashset![0]), "{\"i64-set\":[0]}");
}

#[test]
fn value_string_set() {
    test_value(Value::StringSet(hashset![]), "{\"string-set\":[]}");
    test_value(
        Value::StringSet(hashset!["".to_owned()]),
        "{\"string-set\":[\"\"]}",
    );
    test_value(
        Value::StringSet(hashset!["foo".to_owned()]),
        "{\"string-set\":[\"foo\"]}",
    );
}

#[test]
fn value_uuid_set() {
    test_value(Value::UuidSet(hashset![]), "{\"uuid-set\":[]}");
    test_value(
        Value::UuidSet(hashset![Uuid::nil()]),
        "{\"uuid-set\":[\"00000000-0000-0000-0000-000000000000\"]}",
    );
}

#[test]
fn value_struct() {
    test_value(Value::Struct(hashmap![]), "{\"struct\":{}}");
    test_value(
        Value::Struct(hashmap![0 => Value::None]),
        "{\"struct\":{\"0\":\"none\"}}",
    );
}

#[test]
fn value_enum() {
    test_value(
        Value::Enum {
            variant: 0,
            value: Box::new(Value::None),
        },
        "{\"enum\":{\"variant\":0,\"value\":\"none\"}}",
    );
    test_value(
        Value::Enum {
            variant: 0,
            value: Box::new(Value::U8(0)),
        },
        "{\"enum\":{\"variant\":0,\"value\":{\"u8\":0}}}",
    );
}

#[test]
fn message_connect() {
    test_message(
        Message::Connect(Connect { version: 0 }),
        "{\"connect\":{\"version\":0}}",
    );
}

#[test]
fn message_connect_reply() {
    test_message(
        Message::ConnectReply(ConnectReply::Ok),
        "{\"connect-reply\":\"ok\"}",
    );
    test_message(
        Message::ConnectReply(ConnectReply::VersionMismatch(0)),
        "{\"connect-reply\":{\
            \"version-mismatch\":0\
        }}",
    );
}

#[test]
fn message_shutdown() {
    test_message(Message::Shutdown, "\"shutdown\"");
}

#[test]
fn message_create_object() {
    test_message(
        Message::CreateObject(CreateObject {
            serial: 0,
            uuid: Uuid::nil(),
        }),
        "{\"create-object\":{\
            \"serial\":0,\
            \"uuid\":\"00000000-0000-0000-0000-000000000000\"\
        }}",
    );
}

#[test]
fn message_create_object_reply() {
    test_message(
        Message::CreateObjectReply(CreateObjectReply {
            serial: 0,
            result: CreateObjectResult::Ok(Uuid::nil()),
        }),
        "{\"create-object-reply\":{\
            \"serial\":0,\
            \"result\":{\
                \"ok\":\"00000000-0000-0000-0000-000000000000\"\
        }}}",
    );
    test_message(
        Message::CreateObjectReply(CreateObjectReply {
            serial: 0,
            result: CreateObjectResult::DuplicateObject,
        }),
        "{\"create-object-reply\":{\
            \"serial\":0,\
            \"result\":\"duplicate-object\"\
        }}",
    );
}

#[test]
fn message_destroy_object() {
    test_message(
        Message::DestroyObject(DestroyObject {
            serial: 0,
            cookie: Uuid::nil(),
        }),
        "{\"destroy-object\":{\
            \"serial\":0,\
            \"cookie\":\"00000000-0000-0000-0000-000000000000\"\
        }}",
    );
}

#[test]
fn message_destroy_object_reply() {
    test_message(
        Message::DestroyObjectReply(DestroyObjectReply {
            serial: 0,
            result: DestroyObjectResult::Ok,
        }),
        "{\"destroy-object-reply\":{\
            \"serial\":0,\
            \"result\":\"ok\"\
        }}",
    );
    test_message(
        Message::DestroyObjectReply(DestroyObjectReply {
            serial: 0,
            result: DestroyObjectResult::InvalidObject,
        }),
        "{\"destroy-object-reply\":{\
            \"serial\":0,\
            \"result\":\"invalid-object\"\
        }}",
    );
    test_message(
        Message::DestroyObjectReply(DestroyObjectReply {
            serial: 0,
            result: DestroyObjectResult::ForeignObject,
        }),
        "{\"destroy-object-reply\":{\
            \"serial\":0,\
            \"result\":\"foreign-object\"\
        }}",
    );
}

#[test]
fn message_subscribe_objects() {
    test_message(
        Message::SubscribeObjects(SubscribeObjects { serial: None }),
        "{\"subscribe-objects\":{\
            \"serial\":null\
        }}",
    );
    test_message(
        Message::SubscribeObjects(SubscribeObjects { serial: Some(0) }),
        "{\"subscribe-objects\":{\
            \"serial\":0\
        }}",
    );
}

#[test]
fn message_subscribe_objects_reply() {
    test_message(
        Message::SubscribeObjectsReply(SubscribeObjectsReply { serial: 0 }),
        "{\"subscribe-objects-reply\":{\
            \"serial\":0\
        }}",
    );
}

#[test]
fn message_unsubscribe_objects() {
    test_message(Message::UnsubscribeObjects, "\"unsubscribe-objects\"");
}

#[test]
fn message_object_created_event() {
    test_message(
        Message::ObjectCreatedEvent(ObjectCreatedEvent {
            uuid: Uuid::nil(),
            cookie: Uuid::nil(),
            serial: None,
        }),
        "{\"object-created-event\":{\
            \"uuid\":\"00000000-0000-0000-0000-000000000000\",\
            \"cookie\":\"00000000-0000-0000-0000-000000000000\",\
            \"serial\":null\
        }}",
    );
    test_message(
        Message::ObjectCreatedEvent(ObjectCreatedEvent {
            uuid: Uuid::nil(),
            cookie: Uuid::nil(),
            serial: Some(0),
        }),
        "{\"object-created-event\":{\
            \"uuid\":\"00000000-0000-0000-0000-000000000000\",\
            \"cookie\":\"00000000-0000-0000-0000-000000000000\",\
            \"serial\":0\
        }}",
    );
}

#[test]
fn message_object_destroyed_event() {
    test_message(
        Message::ObjectDestroyedEvent(ObjectDestroyedEvent {
            uuid: Uuid::nil(),
            cookie: Uuid::nil(),
        }),
        "{\"object-destroyed-event\":{\
            \"uuid\":\"00000000-0000-0000-0000-000000000000\",\
            \"cookie\":\"00000000-0000-0000-0000-000000000000\"\
        }}",
    );
}

#[test]
fn message_create_service() {
    test_message(
        Message::CreateService(CreateService {
            serial: 0,
            object_cookie: Uuid::nil(),
            uuid: Uuid::nil(),
        }),
        "{\"create-service\":{\
            \"serial\":0,\
            \"object-cookie\":\"00000000-0000-0000-0000-000000000000\",\
            \"uuid\":\"00000000-0000-0000-0000-000000000000\"\
        }}",
    );
}

#[test]
fn message_create_service_reply() {
    test_message(
        Message::CreateServiceReply(CreateServiceReply {
            serial: 0,
            result: CreateServiceResult::Ok(Uuid::nil()),
        }),
        "{\"create-service-reply\":{\
            \"serial\":0,\
            \"result\":{\
                \"ok\":\"00000000-0000-0000-0000-000000000000\"\
        }}}",
    );
    test_message(
        Message::CreateServiceReply(CreateServiceReply {
            serial: 0,
            result: CreateServiceResult::DuplicateService,
        }),
        "{\"create-service-reply\":{\
            \"serial\":0,\
            \"result\":\"duplicate-service\"\
        }}",
    );
    test_message(
        Message::CreateServiceReply(CreateServiceReply {
            serial: 0,
            result: CreateServiceResult::InvalidObject,
        }),
        "{\"create-service-reply\":{\
            \"serial\":0,\
            \"result\":\"invalid-object\"\
        }}",
    );
    test_message(
        Message::CreateServiceReply(CreateServiceReply {
            serial: 0,
            result: CreateServiceResult::ForeignObject,
        }),
        "{\"create-service-reply\":{\
            \"serial\":0,\
            \"result\":\"foreign-object\"\
        }}",
    );
}

#[test]
fn message_destroy_service() {
    test_message(
        Message::DestroyService(DestroyService {
            serial: 0,
            cookie: Uuid::nil(),
        }),
        "{\"destroy-service\":{\
            \"serial\":0,\
            \"cookie\":\"00000000-0000-0000-0000-000000000000\"\
        }}",
    );
}

#[test]
fn message_destroy_service_reply() {
    test_message(
        Message::DestroyServiceReply(DestroyServiceReply {
            serial: 0,
            result: DestroyServiceResult::Ok,
        }),
        "{\"destroy-service-reply\":{\
            \"serial\":0,\
            \"result\":\"ok\"\
        }}",
    );
    test_message(
        Message::DestroyServiceReply(DestroyServiceReply {
            serial: 0,
            result: DestroyServiceResult::InvalidService,
        }),
        "{\"destroy-service-reply\":{\
            \"serial\":0,\
            \"result\":\"invalid-service\"\
        }}",
    );
    test_message(
        Message::DestroyServiceReply(DestroyServiceReply {
            serial: 0,
            result: DestroyServiceResult::ForeignObject,
        }),
        "{\"destroy-service-reply\":{\
            \"serial\":0,\
            \"result\":\"foreign-object\"\
        }}",
    );
}

#[test]
fn message_subscribe_services() {
    test_message(
        Message::SubscribeServices(SubscribeServices { serial: None }),
        "{\"subscribe-services\":{\
            \"serial\":null\
        }}",
    );
    test_message(
        Message::SubscribeServices(SubscribeServices { serial: Some(0) }),
        "{\"subscribe-services\":{\
            \"serial\":0\
        }}",
    );
}

#[test]
fn message_subscribe_services_reply() {
    test_message(
        Message::SubscribeServicesReply(SubscribeServicesReply { serial: 0 }),
        "{\"subscribe-services-reply\":{\
            \"serial\":0\
        }}",
    );
}

#[test]
fn message_unsubscribe_services() {
    test_message(Message::UnsubscribeServices, "\"unsubscribe-services\"");
}

#[test]
fn message_service_created_event() {
    test_message(
        Message::ServiceCreatedEvent(ServiceCreatedEvent {
            object_uuid: Uuid::nil(),
            object_cookie: Uuid::nil(),
            uuid: Uuid::nil(),
            cookie: Uuid::nil(),
            serial: None,
        }),
        "{\"service-created-event\":{\
            \"object-uuid\":\"00000000-0000-0000-0000-000000000000\",\
            \"object-cookie\":\"00000000-0000-0000-0000-000000000000\",\
            \"uuid\":\"00000000-0000-0000-0000-000000000000\",\
            \"cookie\":\"00000000-0000-0000-0000-000000000000\",\
            \"serial\":null\
        }}",
    );
    test_message(
        Message::ServiceCreatedEvent(ServiceCreatedEvent {
            object_uuid: Uuid::nil(),
            object_cookie: Uuid::nil(),
            uuid: Uuid::nil(),
            cookie: Uuid::nil(),
            serial: Some(0),
        }),
        "{\"service-created-event\":{\
            \"object-uuid\":\"00000000-0000-0000-0000-000000000000\",\
            \"object-cookie\":\"00000000-0000-0000-0000-000000000000\",\
            \"uuid\":\"00000000-0000-0000-0000-000000000000\",\
            \"cookie\":\"00000000-0000-0000-0000-000000000000\",\
            \"serial\":0\
        }}",
    );
}

#[test]
fn message_service_destroyed_event() {
    test_message(
        Message::ServiceDestroyedEvent(ServiceDestroyedEvent {
            object_uuid: Uuid::nil(),
            object_cookie: Uuid::nil(),
            uuid: Uuid::nil(),
            cookie: Uuid::nil(),
        }),
        "{\"service-destroyed-event\":{\
            \"object-uuid\":\"00000000-0000-0000-0000-000000000000\",\
            \"object-cookie\":\"00000000-0000-0000-0000-000000000000\",\
            \"uuid\":\"00000000-0000-0000-0000-000000000000\",\
            \"cookie\":\"00000000-0000-0000-0000-000000000000\"\
        }}",
    );
}

#[test]
fn message_call_function() {
    test_message(
        Message::CallFunction(CallFunction {
            serial: 0,
            service_cookie: Uuid::nil(),
            function: 0,
            args: Value::None,
        }),
        "{\"call-function\":{\
            \"serial\":0,\
            \"service-cookie\":\"00000000-0000-0000-0000-000000000000\",\
            \"function\":0,\
            \"args\":\"none\"\
        }}",
    );
}

#[test]
fn message_call_function_reply() {
    test_message(
        Message::CallFunctionReply(CallFunctionReply {
            serial: 0,
            result: CallFunctionResult::Ok(Value::None),
        }),
        "{\"call-function-reply\":{\
            \"serial\":0,\
            \"result\":{\
                \"ok\":\"none\"\
        }}}",
    );
    test_message(
        Message::CallFunctionReply(CallFunctionReply {
            serial: 0,
            result: CallFunctionResult::Err(Value::None),
        }),
        "{\"call-function-reply\":{\
            \"serial\":0,\
            \"result\":{\
                \"err\":\"none\"\
        }}}",
    );
    test_message(
        Message::CallFunctionReply(CallFunctionReply {
            serial: 0,
            result: CallFunctionResult::Aborted,
        }),
        "{\"call-function-reply\":{\
            \"serial\":0,\
            \"result\":\"aborted\"\
        }}",
    );
    test_message(
        Message::CallFunctionReply(CallFunctionReply {
            serial: 0,
            result: CallFunctionResult::InvalidService,
        }),
        "{\"call-function-reply\":{\
            \"serial\":0,\
            \"result\":\"invalid-service\"\
        }}",
    );
    test_message(
        Message::CallFunctionReply(CallFunctionReply {
            serial: 0,
            result: CallFunctionResult::InvalidFunction,
        }),
        "{\"call-function-reply\":{\
            \"serial\":0,\
            \"result\":\"invalid-function\"\
        }}",
    );
    test_message(
        Message::CallFunctionReply(CallFunctionReply {
            serial: 0,
            result: CallFunctionResult::InvalidArgs,
        }),
        "{\"call-function-reply\":{\
            \"serial\":0,\
            \"result\":\"invalid-args\"\
        }}",
    );
}

#[test]
fn message_subscribe_event() {
    test_message(
        Message::SubscribeEvent(SubscribeEvent {
            serial: None,
            service_cookie: Uuid::nil(),
            event: 0,
        }),
        "{\"subscribe-event\":{\
            \"serial\":null,\
            \"service-cookie\":\"00000000-0000-0000-0000-000000000000\",\
            \"event\":0\
        }}",
    );
    test_message(
        Message::SubscribeEvent(SubscribeEvent {
            serial: Some(0),
            service_cookie: Uuid::nil(),
            event: 0,
        }),
        "{\"subscribe-event\":{\
            \"serial\":0,\
            \"service-cookie\":\"00000000-0000-0000-0000-000000000000\",\
            \"event\":0\
        }}",
    );
}

#[test]
fn message_subscribe_event_reply() {
    test_message(
        Message::SubscribeEventReply(SubscribeEventReply {
            serial: 0,
            result: SubscribeEventResult::Ok,
        }),
        "{\"subscribe-event-reply\":{\
            \"serial\":0,\
            \"result\":\"ok\"\
        }}",
    );
    test_message(
        Message::SubscribeEventReply(SubscribeEventReply {
            serial: 0,
            result: SubscribeEventResult::InvalidService,
        }),
        "{\"subscribe-event-reply\":{\
            \"serial\":0,\
            \"result\":\"invalid-service\"\
        }}",
    );
}

#[test]
fn message_unsubscribe_event() {
    test_message(
        Message::UnsubscribeEvent(UnsubscribeEvent {
            service_cookie: Uuid::nil(),
            event: 0,
        }),
        "{\"unsubscribe-event\":{\
            \"service-cookie\":\"00000000-0000-0000-0000-000000000000\",\
            \"event\":0\
        }}",
    );
}

#[test]
fn message_emit_event() {
    test_message(
        Message::EmitEvent(EmitEvent {
            service_cookie: Uuid::nil(),
            event: 0,
            args: Value::None,
        }),
        "{\"emit-event\":{\
            \"service-cookie\":\"00000000-0000-0000-0000-000000000000\",\
            \"event\":0,\
            \"args\":\"none\"\
        }}",
    );
}
