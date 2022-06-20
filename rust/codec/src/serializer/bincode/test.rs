use super::Bincode;
use crate::{Endian, Serializer};
use aldrin_proto::*;
use maplit::{hashmap, hashset};
use uuid::uuid;

fn test_message(m: &Message, b: &[u8], endian: Endian) {
    let mut ser = Bincode::with_endian(endian);

    let b2 = ser.serialize(m.clone()).unwrap();
    assert_eq!(b, &b2[..]);

    let m2 = ser.deserialize(b2.freeze()).unwrap();
    assert_eq!(*m, m2);
}

fn test_message_le(m: &Message, b: &[u8]) {
    test_message(m, b, Endian::Little);
}

fn test_message_be(m: &Message, b: &[u8]) {
    test_message(m, b, Endian::Big);
}

fn test_value_le(v: &Value, b: &[u8]) {
    let vm = Message::CallFunctionReply(CallFunctionReply {
        serial: 0x12345678,
        result: CallFunctionResult::Ok(v.clone()),
    });
    let mut bm = vec![
        22, // CallFunctionReply
        252, 0x78, 0x56, 0x34, 0x12, // serial
        0,    // Ok
    ];
    bm.extend_from_slice(b);
    test_message_le(&vm, &bm);
}

fn test_value_be(v: &Value, b: &[u8]) {
    let vm = Message::CallFunctionReply(CallFunctionReply {
        serial: 0x12345678,
        result: CallFunctionResult::Ok(v.clone()),
    });
    let mut bm = vec![
        22, // CallFunctionReply
        252, 0x12, 0x34, 0x56, 0x78, // serial
        0,    // Ok
    ];
    bm.extend_from_slice(b);
    test_message_be(&vm, &bm);
}

#[test]
fn message_connect() {
    let m = Message::Connect(Connect {
        version: 0x12345678,
    });
    test_message_le(
        &m,
        &[
            0, // Connect
            252, 0x78, 0x56, 0x34, 0x12, // version
        ],
    );
    test_message_be(
        &m,
        &[
            0, // Connect
            252, 0x12, 0x34, 0x56, 0x78, // version
        ],
    );
}

#[test]
fn message_connect_reply() {
    let m = Message::ConnectReply(ConnectReply::Ok);
    test_message_le(
        &m,
        &[
            1, // ConnectReply
            0, // Ok
        ],
    );
    test_message_be(
        &m,
        &[
            1, // ConnectReply
            0, // Ok
        ],
    );
    let m = Message::ConnectReply(ConnectReply::VersionMismatch(0x12345678));
    test_message_le(
        &m,
        &[
            1, // ConnectReply
            1, // VersionMismatch
            252, 0x78, 0x56, 0x34, 0x12, // version
        ],
    );
    test_message_be(
        &m,
        &[
            1, // ConnectReply
            1, // VersionMismatch
            252, 0x12, 0x34, 0x56, 0x78, // version
        ],
    );
}

#[test]
fn message_shutdown() {
    let m = Message::Shutdown(());
    test_message_le(
        &m,
        &[
            2, // Shutdown
        ],
    );
    test_message_be(
        &m,
        &[
            2, // Shutdown
        ],
    );
}

#[test]
fn message_create_object() {
    let m = Message::CreateObject(CreateObject {
        serial: 0x12345678,
        uuid: uuid!("00112233-4455-6677-8899-aabbccddeeff"),
    });
    test_message_le(
        &m,
        &[
            3, // CreateObject
            252, 0x78, 0x56, 0x34, 0x12, // serial
            16,   // uuid length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // uuid
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // uuid
        ],
    );
    test_message_be(
        &m,
        &[
            3, // CreateObject
            252, 0x12, 0x34, 0x56, 0x78, // serial
            16,   // uuid length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // uuid
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // uuid
        ],
    );
}

#[test]
fn message_create_object_reply() {
    let m = Message::CreateObjectReply(CreateObjectReply {
        serial: 0x12345678,
        result: CreateObjectResult::Ok(uuid!("00112233-4455-6677-8899-aabbccddeeff")),
    });
    test_message_le(
        &m,
        &[
            4, // CreateObjectReply
            252, 0x78, 0x56, 0x34, 0x12, // serial
            0,    // Ok
            16,   // uuid length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // uuid
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // uuid
        ],
    );
    test_message_be(
        &m,
        &[
            4, // CreateObjectReply
            252, 0x12, 0x34, 0x56, 0x78, // serial
            0,    // Ok
            16,   // uuid length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // uuid
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // uuid
        ],
    );
    let m = Message::CreateObjectReply(CreateObjectReply {
        serial: 0x12345678,
        result: CreateObjectResult::DuplicateObject,
    });
    test_message_le(
        &m,
        &[
            4, // CreateObjectReply
            252, 0x78, 0x56, 0x34, 0x12, // serial
            1,    // DuplicateObject
        ],
    );
    test_message_be(
        &m,
        &[
            4, // CreateObjectReply
            252, 0x12, 0x34, 0x56, 0x78, // serial
            1,    // DuplicateObject
        ],
    );
}

#[test]
fn message_destroy_object() {
    let m = Message::DestroyObject(DestroyObject {
        serial: 0x12345678,
        cookie: uuid!("00112233-4455-6677-8899-aabbccddeeff"),
    });
    test_message_le(
        &m,
        &[
            5, // DestroyObject
            252, 0x78, 0x56, 0x34, 0x12, // serial
            16,   // cookie length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // cookie
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // cookie
        ],
    );
    test_message_be(
        &m,
        &[
            5, // DestroyObject
            252, 0x12, 0x34, 0x56, 0x78, // serial
            16,   // cookie length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // cookie
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // cookie
        ],
    );
}

#[test]
fn message_destroy_object_reply() {
    let m = Message::DestroyObjectReply(DestroyObjectReply {
        serial: 0x12345678,
        result: DestroyObjectResult::Ok,
    });
    test_message_le(
        &m,
        &[
            6, // DestroyObjectReply
            252, 0x78, 0x56, 0x34, 0x12, // serial
            0,    // Ok
        ],
    );
    test_message_be(
        &m,
        &[
            6, // DestroyObjectReply
            252, 0x12, 0x34, 0x56, 0x78, // serial
            0,    // Ok
        ],
    );
    let m = Message::DestroyObjectReply(DestroyObjectReply {
        serial: 0x12345678,
        result: DestroyObjectResult::InvalidObject,
    });
    test_message_le(
        &m,
        &[
            6, // DestroyObjectReply
            252, 0x78, 0x56, 0x34, 0x12, // serial
            1,    // InvalidObject
        ],
    );
    test_message_be(
        &m,
        &[
            6, // DestroyObjectReply
            252, 0x12, 0x34, 0x56, 0x78, // serial
            1,    // InvalidObject
        ],
    );
    let m = Message::DestroyObjectReply(DestroyObjectReply {
        serial: 0x12345678,
        result: DestroyObjectResult::ForeignObject,
    });
    test_message_le(
        &m,
        &[
            6, // DestroyObjectReply
            252, 0x78, 0x56, 0x34, 0x12, // serial
            2,    // ForeignObject
        ],
    );
    test_message_be(
        &m,
        &[
            6, // DestroyObjectReply
            252, 0x12, 0x34, 0x56, 0x78, // serial
            2,    // ForeignObject
        ],
    );
}

#[test]
fn message_subscribe_objects() {
    let m = Message::SubscribeObjects(SubscribeObjects { serial: None });
    test_message_le(
        &m,
        &[
            7, // SubscribeObjects
            0, // None
        ],
    );
    test_message_be(
        &m,
        &[
            7, // SubscribeObjects
            0, // None
        ],
    );
    let m = Message::SubscribeObjects(SubscribeObjects {
        serial: Some(0x12345678),
    });
    test_message_le(
        &m,
        &[
            7, // SubscribeObjects
            1, // Some
            252, 0x78, 0x56, 0x34, 0x12, // serial
        ],
    );
    test_message_be(
        &m,
        &[
            7, // SubscribeObjects
            1, // Some
            252, 0x12, 0x34, 0x56, 0x78, // serial
        ],
    );
}

#[test]
fn message_subscribe_objects_reply() {
    let m = Message::SubscribeObjectsReply(SubscribeObjectsReply { serial: 0x12345678 });
    test_message_le(
        &m,
        &[
            8, // SubscribeObjectsReply
            252, 0x78, 0x56, 0x34, 0x12, // serial
        ],
    );
    test_message_be(
        &m,
        &[
            8, // SubscribeObjectsReply
            252, 0x12, 0x34, 0x56, 0x78, // serial
        ],
    );
}

#[test]
fn message_unsubscribe_objects() {
    let m = Message::UnsubscribeObjects(());
    test_message_le(
        &m,
        &[
            9, // UnsubscribeObjects
        ],
    );
    test_message_be(
        &m,
        &[
            9, // UnsubscribeObjects
        ],
    );
}

#[test]
fn message_object_created_event() {
    let m = Message::ObjectCreatedEvent(ObjectCreatedEvent {
        uuid: uuid!("00112233-4455-6677-8899-aabbccddeeff"),
        cookie: uuid!("01122334-4556-6778-899a-abbccddeeff0"),
        serial: None,
    });
    test_message_le(
        &m,
        &[
            10, // ObjectCreatedEvent
            16, // uuid length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // uuid
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // uuid
            16,   // cookie length
            0x01, 0x12, 0x23, 0x34, 0x45, 0x56, 0x67, 0x78, // cookie
            0x89, 0x9a, 0xab, 0xbc, 0xcd, 0xde, 0xef, 0xf0, // cookie
            0,    // None
        ],
    );
    test_message_be(
        &m,
        &[
            10, // ObjectCreatedEvent
            16, // uuid length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // uuid
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // uuid
            16,   // cookie length
            0x01, 0x12, 0x23, 0x34, 0x45, 0x56, 0x67, 0x78, // cookie
            0x89, 0x9a, 0xab, 0xbc, 0xcd, 0xde, 0xef, 0xf0, // cookie
            0,    // None
        ],
    );
    let m = Message::ObjectCreatedEvent(ObjectCreatedEvent {
        uuid: uuid!("00112233-4455-6677-8899-aabbccddeeff"),
        cookie: uuid!("01122334-4556-6778-899a-abbccddeeff0"),
        serial: Some(0x12345678),
    });
    test_message_le(
        &m,
        &[
            10, // ObjectCreatedEvent
            16, // uuid length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // uuid
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // uuid
            16,   // cookie length
            0x01, 0x12, 0x23, 0x34, 0x45, 0x56, 0x67, 0x78, // cookie
            0x89, 0x9a, 0xab, 0xbc, 0xcd, 0xde, 0xef, 0xf0, // cookie
            1,    // Some
            252, 0x78, 0x56, 0x34, 0x12, // serial
        ],
    );
    test_message_be(
        &m,
        &[
            10, // ObjectCreatedEvent
            16, // uuid length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // uuid
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // uuid
            16,   // cookie length
            0x01, 0x12, 0x23, 0x34, 0x45, 0x56, 0x67, 0x78, // cookie
            0x89, 0x9a, 0xab, 0xbc, 0xcd, 0xde, 0xef, 0xf0, // cookie
            1,    // Some
            252, 0x12, 0x34, 0x56, 0x78, // serial
        ],
    );
}

#[test]
fn message_object_destroyed_event() {
    let m = Message::ObjectDestroyedEvent(ObjectDestroyedEvent {
        uuid: uuid!("00112233-4455-6677-8899-aabbccddeeff"),
        cookie: uuid!("01122334-4556-6778-899a-abbccddeeff0"),
    });
    test_message_le(
        &m,
        &[
            11, // ObjectDestroyedEvent
            16, // uuid length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // uuid
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // uuid
            16,   // cookie length
            0x01, 0x12, 0x23, 0x34, 0x45, 0x56, 0x67, 0x78, // cookie
            0x89, 0x9a, 0xab, 0xbc, 0xcd, 0xde, 0xef, 0xf0, // cookie
        ],
    );
    test_message_be(
        &m,
        &[
            11, // ObjectDestroyedEvent
            16, // uuid length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // uuid
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // uuid
            16,   // cookie length
            0x01, 0x12, 0x23, 0x34, 0x45, 0x56, 0x67, 0x78, // cookie
            0x89, 0x9a, 0xab, 0xbc, 0xcd, 0xde, 0xef, 0xf0, // cookie
        ],
    );
}

#[test]
fn message_create_service() {
    let m = Message::CreateService(CreateService {
        serial: 0x12345678,
        object_cookie: uuid!("00112233-4455-6677-8899-aabbccddeeff"),
        uuid: uuid!("01122334-4556-6778-899a-abbccddeeff0"),
        version: 0x01234567,
    });
    test_message_le(
        &m,
        &[
            12, // CreateService
            252, 0x78, 0x56, 0x34, 0x12, // serial
            16,   // object_cookie length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // object_cookie
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // object_cookie
            16,   // uuid length
            0x01, 0x12, 0x23, 0x34, 0x45, 0x56, 0x67, 0x78, // uuid
            0x89, 0x9a, 0xab, 0xbc, 0xcd, 0xde, 0xef, 0xf0, // uuid
            252, 0x67, 0x45, 0x23, 0x01, // version
        ],
    );
    test_message_be(
        &m,
        &[
            12, // CreateService
            252, 0x12, 0x34, 0x56, 0x78, // serial
            16,   // object_cookie length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // object_cookie
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // object_cookie
            16,   // uuid length
            0x01, 0x12, 0x23, 0x34, 0x45, 0x56, 0x67, 0x78, // uuid
            0x89, 0x9a, 0xab, 0xbc, 0xcd, 0xde, 0xef, 0xf0, // uuid
            252, 0x01, 0x23, 0x45, 0x67, // version
        ],
    );
}

#[test]
fn message_create_service_reply() {
    let m = Message::CreateServiceReply(CreateServiceReply {
        serial: 0x12345678,
        result: CreateServiceResult::Ok(uuid!("00112233-4455-6677-8899-aabbccddeeff")),
    });
    test_message_le(
        &m,
        &[
            13, // CreateServiceReply
            252, 0x78, 0x56, 0x34, 0x12, // serial
            0,    // Ok
            16,   // cookie length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // cookie
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // cookie
        ],
    );
    test_message_be(
        &m,
        &[
            13, // CreateServiceReply
            252, 0x12, 0x34, 0x56, 0x78, // serial
            0,    // Ok
            16,   // cookie length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // cookie
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // cookie
        ],
    );
    let m = Message::CreateServiceReply(CreateServiceReply {
        serial: 0x12345678,
        result: CreateServiceResult::DuplicateService,
    });
    test_message_le(
        &m,
        &[
            13, // CreateServiceReply
            252, 0x78, 0x56, 0x34, 0x12, // serial
            1,    // DuplicateService
        ],
    );
    test_message_be(
        &m,
        &[
            13, // CreateServiceReply
            252, 0x12, 0x34, 0x56, 0x78, // serial
            1,    // DuplicateService
        ],
    );
    let m = Message::CreateServiceReply(CreateServiceReply {
        serial: 0x12345678,
        result: CreateServiceResult::InvalidObject,
    });
    test_message_le(
        &m,
        &[
            13, // CreateServiceReply
            252, 0x78, 0x56, 0x34, 0x12, // serial
            2,    // InvalidObject
        ],
    );
    test_message_be(
        &m,
        &[
            13, // CreateServiceReply
            252, 0x12, 0x34, 0x56, 0x78, // serial
            2,    // InvalidObject
        ],
    );
    let m = Message::CreateServiceReply(CreateServiceReply {
        serial: 0x12345678,
        result: CreateServiceResult::ForeignObject,
    });
    test_message_le(
        &m,
        &[
            13, // CreateServiceReply
            252, 0x78, 0x56, 0x34, 0x12, // serial
            3,    // ForeignObject
        ],
    );
    test_message_be(
        &m,
        &[
            13, // CreateServiceReply
            252, 0x12, 0x34, 0x56, 0x78, // serial
            3,    // ForeignObject
        ],
    );
}

#[test]
fn message_destroy_service() {
    let m = Message::DestroyService(DestroyService {
        serial: 0x12345678,
        cookie: uuid!("00112233-4455-6677-8899-aabbccddeeff"),
    });
    test_message_le(
        &m,
        &[
            14, // DestroyService
            252, 0x78, 0x56, 0x34, 0x12, // serial
            16,   // cookie length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // cookie
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // cookie
        ],
    );
    test_message_be(
        &m,
        &[
            14, // DestroyService
            252, 0x12, 0x34, 0x56, 0x78, // serial
            16,   // cookie length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // cookie
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // cookie
        ],
    );
}

#[test]
fn message_destroy_service_reply() {
    let m = Message::DestroyServiceReply(DestroyServiceReply {
        serial: 0x12345678,
        result: DestroyServiceResult::Ok,
    });
    test_message_le(
        &m,
        &[
            15, // DestroyServiceReply
            252, 0x78, 0x56, 0x34, 0x12, // serial
            0,    // Ok
        ],
    );
    test_message_be(
        &m,
        &[
            15, // DestroyServiceReply
            252, 0x12, 0x34, 0x56, 0x78, // serial
            0,    // Ok
        ],
    );
    let m = Message::DestroyServiceReply(DestroyServiceReply {
        serial: 0x12345678,
        result: DestroyServiceResult::InvalidService,
    });
    test_message_le(
        &m,
        &[
            15, // DestroyServiceReply
            252, 0x78, 0x56, 0x34, 0x12, // serial
            1,    // InvalidService
        ],
    );
    test_message_be(
        &m,
        &[
            15, // DestroyServiceReply
            252, 0x12, 0x34, 0x56, 0x78, // serial
            1,    // InvalidService
        ],
    );
    let m = Message::DestroyServiceReply(DestroyServiceReply {
        serial: 0x12345678,
        result: DestroyServiceResult::ForeignObject,
    });
    test_message_le(
        &m,
        &[
            15, // DestroyServiceReply
            252, 0x78, 0x56, 0x34, 0x12, // serial
            2,    // ForeignObject
        ],
    );
    test_message_be(
        &m,
        &[
            15, // DestroyServiceReply
            252, 0x12, 0x34, 0x56, 0x78, // serial
            2,    // ForeignObject
        ],
    );
}

#[test]
fn message_subscribe_services() {
    let m = Message::SubscribeServices(SubscribeServices { serial: None });
    test_message_le(
        &m,
        &[
            16, // SubscribeServices
            0,  // None
        ],
    );
    test_message_be(
        &m,
        &[
            16, // SubscribeServices
            0,  // None
        ],
    );
    let m = Message::SubscribeServices(SubscribeServices {
        serial: Some(0x12345678),
    });
    test_message_le(
        &m,
        &[
            16, // SubscribeServices
            1,  // Some
            252, 0x78, 0x56, 0x34, 0x12, // serial
        ],
    );
    test_message_be(
        &m,
        &[
            16, // SubscribeServices
            1,  // Some
            252, 0x12, 0x34, 0x56, 0x78, // serial
        ],
    );
}

#[test]
fn message_subscribe_services_reply() {
    let m = Message::SubscribeServicesReply(SubscribeServicesReply { serial: 0x12345678 });
    test_message_le(
        &m,
        &[
            17, // SubscribeServicesReply
            252, 0x78, 0x56, 0x34, 0x12, // serial
        ],
    );
    test_message_be(
        &m,
        &[
            17, // SubscribeServicesReply
            252, 0x12, 0x34, 0x56, 0x78, // serial
        ],
    );
}

#[test]
fn message_unsubscribe_services() {
    let m = Message::UnsubscribeServices(());
    test_message_le(
        &m,
        &[
            18, // UnsubscribeServices
        ],
    );
    test_message_be(
        &m,
        &[
            18, // UnsubscribeServices
        ],
    );
}

#[test]
fn message_service_created_event() {
    let m = Message::ServiceCreatedEvent(ServiceCreatedEvent {
        object_uuid: uuid!("00112233-4455-6677-8899-aabbccddeeff"),
        object_cookie: uuid!("01122334-4556-6778-899a-abbccddeeff0"),
        uuid: uuid!("02132435-4657-6879-8a9b-acbdcedfe0f1"),
        cookie: uuid!("03142536-4758-697a-8b9c-adbecfd0e1f2"),
        serial: None,
    });
    test_message_le(
        &m,
        &[
            19, // ServiceCreatedEvent
            16, // object_uuid length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // object_uuid
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // object_uuid
            16,   // object_cookie length
            0x01, 0x12, 0x23, 0x34, 0x45, 0x56, 0x67, 0x78, // object_cookie
            0x89, 0x9a, 0xab, 0xbc, 0xcd, 0xde, 0xef, 0xf0, // object_cookie
            16,   // uuid length
            0x02, 0x13, 0x24, 0x35, 0x46, 0x57, 0x68, 0x79, // uuid
            0x8a, 0x9b, 0xac, 0xbd, 0xce, 0xdf, 0xe0, 0xf1, // uuid
            16,   // cookie length
            0x03, 0x14, 0x25, 0x36, 0x47, 0x58, 0x69, 0x7a, // cookie
            0x8b, 0x9c, 0xad, 0xbe, 0xcf, 0xd0, 0xe1, 0xf2, // cookie
            0,    // None
        ],
    );
    test_message_be(
        &m,
        &[
            19, // ServiceCreatedEvent
            16, // object_uuid length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // object_uuid
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // object_uuid
            16,   // object_cookie length
            0x01, 0x12, 0x23, 0x34, 0x45, 0x56, 0x67, 0x78, // object_cookie
            0x89, 0x9a, 0xab, 0xbc, 0xcd, 0xde, 0xef, 0xf0, // object_cookie
            16,   // uuid length
            0x02, 0x13, 0x24, 0x35, 0x46, 0x57, 0x68, 0x79, // uuid
            0x8a, 0x9b, 0xac, 0xbd, 0xce, 0xdf, 0xe0, 0xf1, // uuid
            16,   // cookie length
            0x03, 0x14, 0x25, 0x36, 0x47, 0x58, 0x69, 0x7a, // cookie
            0x8b, 0x9c, 0xad, 0xbe, 0xcf, 0xd0, 0xe1, 0xf2, // cookie
            0,    // None
        ],
    );
    let m = Message::ServiceCreatedEvent(ServiceCreatedEvent {
        object_uuid: uuid!("00112233-4455-6677-8899-aabbccddeeff"),
        object_cookie: uuid!("01122334-4556-6778-899a-abbccddeeff0"),
        uuid: uuid!("02132435-4657-6879-8a9b-acbdcedfe0f1"),
        cookie: uuid!("03142536-4758-697a-8b9c-adbecfd0e1f2"),
        serial: Some(0x12345678),
    });
    test_message_le(
        &m,
        &[
            19, // ServiceCreatedEvent
            16, // object_uuid length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // object_uuid
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // object_uuid
            16,   // object_cookie length
            0x01, 0x12, 0x23, 0x34, 0x45, 0x56, 0x67, 0x78, // object_cookie
            0x89, 0x9a, 0xab, 0xbc, 0xcd, 0xde, 0xef, 0xf0, // object_cookie
            16,   // uuid length
            0x02, 0x13, 0x24, 0x35, 0x46, 0x57, 0x68, 0x79, // uuid
            0x8a, 0x9b, 0xac, 0xbd, 0xce, 0xdf, 0xe0, 0xf1, // uuid
            16,   // cookie length
            0x03, 0x14, 0x25, 0x36, 0x47, 0x58, 0x69, 0x7a, // cookie
            0x8b, 0x9c, 0xad, 0xbe, 0xcf, 0xd0, 0xe1, 0xf2, // cookie
            1,    // Some
            252, 0x78, 0x56, 0x34, 0x12, // serial
        ],
    );
    test_message_be(
        &m,
        &[
            19, // ServiceCreatedEvent
            16, // object_uuid length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // object_uuid
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // object_uuid
            16,   // object_cookie length
            0x01, 0x12, 0x23, 0x34, 0x45, 0x56, 0x67, 0x78, // object_cookie
            0x89, 0x9a, 0xab, 0xbc, 0xcd, 0xde, 0xef, 0xf0, // object_cookie
            16,   // uuid length
            0x02, 0x13, 0x24, 0x35, 0x46, 0x57, 0x68, 0x79, // uuid
            0x8a, 0x9b, 0xac, 0xbd, 0xce, 0xdf, 0xe0, 0xf1, // uuid
            16,   // cookie length
            0x03, 0x14, 0x25, 0x36, 0x47, 0x58, 0x69, 0x7a, // cookie
            0x8b, 0x9c, 0xad, 0xbe, 0xcf, 0xd0, 0xe1, 0xf2, // cookie
            1,    // Some
            252, 0x12, 0x34, 0x56, 0x78, // serial
        ],
    );
}

#[test]
fn message_service_destroyed_event() {
    let m = Message::ServiceDestroyedEvent(ServiceDestroyedEvent {
        object_uuid: uuid!("00112233-4455-6677-8899-aabbccddeeff"),
        object_cookie: uuid!("01122334-4556-6778-899a-abbccddeeff0"),
        uuid: uuid!("02132435-4657-6879-8a9b-acbdcedfe0f1"),
        cookie: uuid!("03142536-4758-697a-8b9c-adbecfd0e1f2"),
    });
    test_message_le(
        &m,
        &[
            20, // ServiceDestroyedEvent
            16, // object_uuid length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // object_uuid
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // object_uuid
            16,   // object_cookie length
            0x01, 0x12, 0x23, 0x34, 0x45, 0x56, 0x67, 0x78, // object_cookie
            0x89, 0x9a, 0xab, 0xbc, 0xcd, 0xde, 0xef, 0xf0, // object_cookie
            16,   // uuid length
            0x02, 0x13, 0x24, 0x35, 0x46, 0x57, 0x68, 0x79, // uuid
            0x8a, 0x9b, 0xac, 0xbd, 0xce, 0xdf, 0xe0, 0xf1, // uuid
            16,   // cookie length
            0x03, 0x14, 0x25, 0x36, 0x47, 0x58, 0x69, 0x7a, // cookie
            0x8b, 0x9c, 0xad, 0xbe, 0xcf, 0xd0, 0xe1, 0xf2, // cookie
        ],
    );
    test_message_be(
        &m,
        &[
            20, // ServiceDestroyedEvent
            16, // object_uuid length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // object_uuid
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // object_uuid
            16,   // object_cookie length
            0x01, 0x12, 0x23, 0x34, 0x45, 0x56, 0x67, 0x78, // object_cookie
            0x89, 0x9a, 0xab, 0xbc, 0xcd, 0xde, 0xef, 0xf0, // object_cookie
            16,   // uuid length
            0x02, 0x13, 0x24, 0x35, 0x46, 0x57, 0x68, 0x79, // uuid
            0x8a, 0x9b, 0xac, 0xbd, 0xce, 0xdf, 0xe0, 0xf1, // uuid
            16,   // cookie length
            0x03, 0x14, 0x25, 0x36, 0x47, 0x58, 0x69, 0x7a, // cookie
            0x8b, 0x9c, 0xad, 0xbe, 0xcf, 0xd0, 0xe1, 0xf2, // cookie
        ],
    );
}

#[test]
fn message_call_function() {
    let m = Message::CallFunction(CallFunction {
        serial: 0x12345678,
        service_cookie: uuid!("00112233-4455-6677-8899-aabbccddeeff"),
        function: 0x87654321,
        args: Value::None,
    });
    test_message_le(
        &m,
        &[
            21, // CallFunction
            252, 0x78, 0x56, 0x34, 0x12, // serial
            16,   // service_cookie length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // service_cookie
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // service_cookie
            252, 0x21, 0x43, 0x65, 0x87, // function
            0,    // args
        ],
    );
    test_message_be(
        &m,
        &[
            21, // CallFunction
            252, 0x12, 0x34, 0x56, 0x78, // serial
            16,   // service_cookie length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // service_cookie
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // service_cookie
            252, 0x87, 0x65, 0x43, 0x21, // function
            0,    // args
        ],
    );
}

#[test]
fn message_call_function_reply() {
    let m = Message::CallFunctionReply(CallFunctionReply {
        serial: 0x12345678,
        result: CallFunctionResult::Ok(Value::None),
    });
    test_message_le(
        &m,
        &[
            22, // CallFunctionReply
            252, 0x78, 0x56, 0x34, 0x12, // serial
            0,    // Ok
            0,    // value
        ],
    );
    test_message_be(
        &m,
        &[
            22, // CallFunctionReply
            252, 0x12, 0x34, 0x56, 0x78, // serial
            0,    // Ok
            0,    // value
        ],
    );
    let m = Message::CallFunctionReply(CallFunctionReply {
        serial: 0x12345678,
        result: CallFunctionResult::Err(Value::None),
    });
    test_message_le(
        &m,
        &[
            22, // CallFunctionReply
            252, 0x78, 0x56, 0x34, 0x12, // serial
            1,    // Err
            0,    // value
        ],
    );
    test_message_be(
        &m,
        &[
            22, // CallFunctionReply
            252, 0x12, 0x34, 0x56, 0x78, // serial
            1,    // Err
            0,    // value
        ],
    );
    let m = Message::CallFunctionReply(CallFunctionReply {
        serial: 0x12345678,
        result: CallFunctionResult::Aborted,
    });
    test_message_le(
        &m,
        &[
            22, // CallFunctionReply
            252, 0x78, 0x56, 0x34, 0x12, // serial
            2,    // Aborted
        ],
    );
    test_message_be(
        &m,
        &[
            22, // CallFunctionReply
            252, 0x12, 0x34, 0x56, 0x78, // serial
            2,    // Aborted
        ],
    );
    let m = Message::CallFunctionReply(CallFunctionReply {
        serial: 0x12345678,
        result: CallFunctionResult::InvalidService,
    });
    test_message_le(
        &m,
        &[
            22, // CallFunctionReply
            252, 0x78, 0x56, 0x34, 0x12, // serial
            3,    // InvalidService
        ],
    );
    test_message_be(
        &m,
        &[
            22, // CallFunctionReply
            252, 0x12, 0x34, 0x56, 0x78, // serial
            3,    // InvalidService
        ],
    );
    let m = Message::CallFunctionReply(CallFunctionReply {
        serial: 0x12345678,
        result: CallFunctionResult::InvalidFunction,
    });
    test_message_le(
        &m,
        &[
            22, // CallFunctionReply
            252, 0x78, 0x56, 0x34, 0x12, // serial
            4,    // InvalidFunction
        ],
    );
    test_message_be(
        &m,
        &[
            22, // CallFunctionReply
            252, 0x12, 0x34, 0x56, 0x78, // serial
            4,    // InvalidFunction
        ],
    );
    let m = Message::CallFunctionReply(CallFunctionReply {
        serial: 0x12345678,
        result: CallFunctionResult::InvalidArgs,
    });
    test_message_le(
        &m,
        &[
            22, // CallFunctionReply
            252, 0x78, 0x56, 0x34, 0x12, // serial
            5,    // InvalidArgs
        ],
    );
    test_message_be(
        &m,
        &[
            22, // CallFunctionReply
            252, 0x12, 0x34, 0x56, 0x78, // serial
            5,    // InvalidArgs
        ],
    );
}

#[test]
fn message_subscribe_event() {
    let m = Message::SubscribeEvent(SubscribeEvent {
        serial: None,
        service_cookie: uuid!("00112233-4455-6677-8899-aabbccddeeff"),
        event: 0x87654321,
    });
    test_message_le(
        &m,
        &[
            23, // SubscribeEvent
            0,  // None
            16, // service_cookie length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // service_cookie
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // service_cookie
            252, 0x21, 0x43, 0x65, 0x87, // event
        ],
    );
    test_message_be(
        &m,
        &[
            23, // SubscribeEvent
            0,  // None
            16, // service_cookie length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // service_cookie
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // service_cookie
            252, 0x87, 0x65, 0x43, 0x21, // event
        ],
    );
    let m = Message::SubscribeEvent(SubscribeEvent {
        serial: Some(0x12345678),
        service_cookie: uuid!("00112233-4455-6677-8899-aabbccddeeff"),
        event: 0x87654321,
    });
    test_message_le(
        &m,
        &[
            23, // SubscribeEvent
            1,  // Some
            252, 0x78, 0x56, 0x34, 0x12, // serial
            16,   // service_cookie length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // service_cookie
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // service_cookie
            252, 0x21, 0x43, 0x65, 0x87, // event
        ],
    );
    test_message_be(
        &m,
        &[
            23, // SubscribeEvent
            1,  // Some
            252, 0x12, 0x34, 0x56, 0x78, // serial
            16,   // service_cookie length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // service_cookie
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // service_cookie
            252, 0x87, 0x65, 0x43, 0x21, // event
        ],
    );
}

#[test]
fn message_subscribe_event_reply() {
    let m = Message::SubscribeEventReply(SubscribeEventReply {
        serial: 0x12345678,
        result: SubscribeEventResult::Ok,
    });
    test_message_le(
        &m,
        &[
            24, // SubscribeEventReply
            252, 0x78, 0x56, 0x34, 0x12, // serial
            0,    // Ok
        ],
    );
    test_message_be(
        &m,
        &[
            24, // SubscribeEventReply
            252, 0x12, 0x34, 0x56, 0x78, // serial
            0,    // Ok
        ],
    );
    let m = Message::SubscribeEventReply(SubscribeEventReply {
        serial: 0x12345678,
        result: SubscribeEventResult::InvalidService,
    });
    test_message_le(
        &m,
        &[
            24, // SubscribeEventReply
            252, 0x78, 0x56, 0x34, 0x12, // serial
            1,    // InvalidService
        ],
    );
    test_message_be(
        &m,
        &[
            24, // SubscribeEventReply
            252, 0x12, 0x34, 0x56, 0x78, // serial
            1,    // InvalidService
        ],
    );
}

#[test]
fn message_unsubscribe_event() {
    let m = Message::UnsubscribeEvent(UnsubscribeEvent {
        service_cookie: uuid!("00112233-4455-6677-8899-aabbccddeeff"),
        event: 0x87654321,
    });
    test_message_le(
        &m,
        &[
            25, // UnsubscribeEvent
            16, // service_cookie length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // service_cookie
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // service_cookie
            252, 0x21, 0x43, 0x65, 0x87, // event
        ],
    );
    test_message_be(
        &m,
        &[
            25, // UnsubscribeEvent
            16, // uuid length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // service_cookie
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // service_cookie
            252, 0x87, 0x65, 0x43, 0x21, // event
        ],
    );
}

#[test]
fn message_emit_event() {
    let m = Message::EmitEvent(EmitEvent {
        service_cookie: uuid!("00112233-4455-6677-8899-aabbccddeeff"),
        event: 0x87654321,
        args: Value::None,
    });
    test_message_le(
        &m,
        &[
            26, // EmitEvent
            16, // service_cookie length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // service_cookie
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // service_cookie
            252, 0x21, 0x43, 0x65, 0x87, // event
            0,    // args
        ],
    );
    test_message_be(
        &m,
        &[
            26, // EmitEvent
            16, // service_cookie length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // service_cookie
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // service_cookie
            252, 0x87, 0x65, 0x43, 0x21, // event
            0,    // args
        ],
    );
}

#[test]
fn message_query_object() {
    let m = Message::QueryObject(QueryObject {
        serial: 0x12345678,
        uuid: uuid!("00112233-4455-6677-8899-aabbccddeeff"),
        with_services: true,
    });
    test_message_le(
        &m,
        &[
            27, // QueryObject
            252, 0x78, 0x56, 0x34, 0x12, // serial
            16,   // uuid length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // uuid
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // uuid
            1,    // with_services
        ],
    );
    test_message_be(
        &m,
        &[
            27, // QueryObject
            252, 0x12, 0x34, 0x56, 0x78, // serial
            16,   // uuid length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // uuid
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // uuid
            1,    // with_services
        ],
    );
}

#[test]
fn message_query_object_reply() {
    let m = Message::QueryObjectReply(QueryObjectReply {
        serial: 0x12345678,
        result: QueryObjectResult::Cookie(uuid!("00112233-4455-6677-8899-aabbccddeeff")),
    });
    test_message_le(
        &m,
        &[
            28, // QueryObjectReply
            252, 0x78, 0x56, 0x34, 0x12, // serial
            0,    // Cookie
            16,   // uuid length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // uuid
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // uuid
        ],
    );
    test_message_be(
        &m,
        &[
            28, // QueryObjectReply
            252, 0x12, 0x34, 0x56, 0x78, // serial
            0,    // Cookie
            16,   // uuid length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // uuid
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // uuid
        ],
    );
    let m = Message::QueryObjectReply(QueryObjectReply {
        serial: 0x12345678,
        result: QueryObjectResult::Service {
            uuid: uuid!("00112233-4455-6677-8899-aabbccddeeff"),
            cookie: uuid!("01122334-4556-6778-899a-abbccddeeff0"),
        },
    });
    test_message_le(
        &m,
        &[
            28, // QueryObjectReply
            252, 0x78, 0x56, 0x34, 0x12, // serial
            1,    // Service
            16,   // uuid length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // uuid
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // uuid
            16,   // cookie length
            0x01, 0x12, 0x23, 0x34, 0x45, 0x56, 0x67, 0x78, // cookie
            0x89, 0x9a, 0xab, 0xbc, 0xcd, 0xde, 0xef, 0xf0, // cookie
        ],
    );
    test_message_be(
        &m,
        &[
            28, // QueryObjectReply
            252, 0x12, 0x34, 0x56, 0x78, // serial
            1,    // Cookie
            16,   // uuid length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // uuid
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // uuid
            16,   // cookie length
            0x01, 0x12, 0x23, 0x34, 0x45, 0x56, 0x67, 0x78, // cookie
            0x89, 0x9a, 0xab, 0xbc, 0xcd, 0xde, 0xef, 0xf0, // cookie
        ],
    );
    let m = Message::QueryObjectReply(QueryObjectReply {
        serial: 0x12345678,
        result: QueryObjectResult::Done,
    });
    test_message_le(
        &m,
        &[
            28, // QueryObjectReply
            252, 0x78, 0x56, 0x34, 0x12, // serial
            2,    // Done
        ],
    );
    test_message_be(
        &m,
        &[
            28, // QueryObjectReply
            252, 0x12, 0x34, 0x56, 0x78, // serial
            2,    // Done
        ],
    );
    let m = Message::QueryObjectReply(QueryObjectReply {
        serial: 0x12345678,
        result: QueryObjectResult::InvalidObject,
    });
    test_message_le(
        &m,
        &[
            28, // QueryObjectReply
            252, 0x78, 0x56, 0x34, 0x12, // serial
            3,    // InvalidObject
        ],
    );
    test_message_be(
        &m,
        &[
            28, // QueryObjectReply
            252, 0x12, 0x34, 0x56, 0x78, // serial
            3,    // InvalidObject
        ],
    );
}

#[test]
fn message_query_service_version() {
    let m = Message::QueryServiceVersion(QueryServiceVersion {
        serial: 0x12345678,
        cookie: uuid!("00112233-4455-6677-8899-aabbccddeeff"),
    });
    test_message_le(
        &m,
        &[
            29, // QueryServiceVersion
            252, 0x78, 0x56, 0x34, 0x12, // serial
            16,   // cookie length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // cookie
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // cookie
        ],
    );
    test_message_be(
        &m,
        &[
            29, // QueryServiceVersion
            252, 0x12, 0x34, 0x56, 0x78, // serial
            16,   // cookie length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // cookie
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // cookie
        ],
    );
}

#[test]
fn message_query_service_version_reply() {
    let m = Message::QueryServiceVersionReply(QueryServiceVersionReply {
        serial: 0x12345678,
        result: QueryServiceVersionResult::Ok(0x01234567),
    });
    test_message_le(
        &m,
        &[
            30, // QueryServiceVersionReply
            252, 0x78, 0x56, 0x34, 0x12, // serial
            0,    // Ok
            252, 0x67, 0x45, 0x23, 0x01, // version
        ],
    );
    test_message_be(
        &m,
        &[
            30, // QueryServiceVersionReply
            252, 0x12, 0x34, 0x56, 0x78, // serial
            0,    // Ok
            252, 0x01, 0x23, 0x45, 0x67, // version
        ],
    );
    let m = Message::QueryServiceVersionReply(QueryServiceVersionReply {
        serial: 0x12345678,
        result: QueryServiceVersionResult::InvalidService,
    });
    test_message_le(
        &m,
        &[
            30, // QueryServiceVersionReply
            252, 0x78, 0x56, 0x34, 0x12, // serial
            1,    // InvalidService
        ],
    );
    test_message_be(
        &m,
        &[
            30, // QueryServiceVersionReply
            252, 0x12, 0x34, 0x56, 0x78, // serial
            1,    // InvalidService
        ],
    );
}

#[test]
fn value_none() {
    let v = Value::None;
    test_value_le(
        &v,
        &[
            0, // None
        ],
    );
    test_value_be(
        &v,
        &[
            0, // None
        ],
    );
}

#[test]
fn value_bool() {
    let v = Value::Bool(false);
    test_value_le(
        &v,
        &[
            1, // Bool
            0, // false
        ],
    );
    test_value_be(
        &v,
        &[
            1, // Bool
            0, // false
        ],
    );
    let v = Value::Bool(true);
    test_value_le(
        &v,
        &[
            1, // Bool
            1, // true
        ],
    );
    test_value_be(
        &v,
        &[
            1, // Bool
            1, // true
        ],
    );
}

#[test]
fn value_u8() {
    let v = Value::U8(0x12);
    test_value_le(
        &v,
        &[
            2,    // U8
            0x12, // value
        ],
    );
    test_value_be(
        &v,
        &[
            2,    // U8
            0x12, // value
        ],
    );
}

#[test]
fn value_i8() {
    let v = Value::I8(0x12);
    test_value_le(
        &v,
        &[
            3,    // I8
            0x12, // value
        ],
    );
    test_value_be(
        &v,
        &[
            3,    // I8
            0x12, // value
        ],
    );
}

#[test]
fn value_u16() {
    let v = Value::U16(0x1234);
    test_value_le(
        &v,
        &[
            4, // U16
            251, 0x34, 0x12, // value
        ],
    );
    test_value_be(
        &v,
        &[
            4, // U16
            251, 0x12, 0x34, // value
        ],
    );
}

#[test]
fn value_i16() {
    let v = Value::I16(0x1234);
    test_value_le(
        &v,
        &[
            5, // I16
            251, 0x68, 0x24, // value
        ],
    );
    test_value_be(
        &v,
        &[
            5, // I16
            251, 0x24, 0x68, // value
        ],
    );
}

#[test]
fn value_u32() {
    let v = Value::U32(0x12345678);
    test_value_le(
        &v,
        &[
            6, // U32
            252, 0x78, 0x56, 0x34, 0x12, // value
        ],
    );
    test_value_be(
        &v,
        &[
            6, // U32
            252, 0x12, 0x34, 0x56, 0x78, // value
        ],
    );
}

#[test]
fn value_i32() {
    let v = Value::I32(0x12345678);
    test_value_le(
        &v,
        &[
            7, // I32
            252, 0xf0, 0xac, 0x68, 0x24, // value
        ],
    );
    test_value_be(
        &v,
        &[
            7, // I32
            252, 0x24, 0x68, 0xac, 0xf0, // value
        ],
    );
}

#[test]
fn value_u64() {
    let v = Value::U64(0x123456789abcdef0);
    test_value_le(
        &v,
        &[
            8, // U64
            253, 0xf0, 0xde, 0xbc, 0x9a, 0x78, 0x56, 0x34, 0x12, // value
        ],
    );
    test_value_be(
        &v,
        &[
            8, // U64
            253, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, // value
        ],
    );
}

#[test]
fn value_i64() {
    let v = Value::I64(0x123456789abcdef0);
    test_value_le(
        &v,
        &[
            9, // I64
            253, 0xe0, 0xbd, 0x79, 0x35, 0xf1, 0xac, 0x68, 0x24, // value
        ],
    );
    test_value_be(
        &v,
        &[
            9, // I64
            253, 0x24, 0x68, 0xac, 0xf1, 0x35, 0x79, 0xbd, 0xe0, // value
        ],
    );
}

#[test]
fn value_f32() {
    let v = Value::F32(1.7378244e34);
    test_value_le(
        &v,
        &[
            10, // F32
            0x12, 0x34, 0x56, 0x78, // value
        ],
    );
    test_value_be(
        &v,
        &[
            10, // F32
            0x78, 0x56, 0x34, 0x12, // value
        ],
    );
}

#[test]
fn value_f64() {
    let v = Value::F64(-4.886459655043775e235);
    test_value_le(
        &v,
        &[
            11, // F64
            0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, // value
        ],
    );
    test_value_be(
        &v,
        &[
            11, // F64
            0xf0, 0xde, 0xbc, 0x9a, 0x78, 0x56, 0x34, 0x12, // value
        ],
    );
}

#[test]
fn value_string() {
    let v = Value::String("".to_owned());
    test_value_le(
        &v,
        &[
            12, // String
            0,  // length
        ],
    );
    test_value_be(
        &v,
        &[
            12, // String
            0,  // length
        ],
    );
    let v = Value::String("aldrin".to_owned());
    test_value_le(
        &v,
        &[
            12, // String
            6,  // length
            b'a', b'l', b'd', b'r', b'i', b'n', // value
        ],
    );
    test_value_be(
        &v,
        &[
            12, // String
            6,  // length
            b'a', b'l', b'd', b'r', b'i', b'n', // value
        ],
    );
}

#[test]
fn value_uuid() {
    let v = Value::Uuid(uuid!("00112233-4455-6677-8899-aabbccddeeff"));
    test_value_le(
        &v,
        &[
            13, // Uuid
            16, // uuid length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // value
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // value
        ],
    );
    test_value_be(
        &v,
        &[
            13, // Uuid
            16, // uuid length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // value
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // value
        ],
    );
}

#[test]
fn value_object_id() {
    let v = Value::ObjectId(ObjectId {
        uuid: uuid!("00112233-4455-6677-8899-aabbccddeeff"),
        cookie: uuid!("01122334-4556-6778-899a-abbccddeeff0"),
    });
    test_value_le(
        &v,
        &[
            14, // ObjectId
            16, // uuid length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // uuid
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // uuid
            16,   // cookie length
            0x01, 0x12, 0x23, 0x34, 0x45, 0x56, 0x67, 0x78, // cookie
            0x89, 0x9a, 0xab, 0xbc, 0xcd, 0xde, 0xef, 0xf0, // cookie
        ],
    );
    test_value_be(
        &v,
        &[
            14, // ObjectId
            16, // uuid length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // uuid
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // uuid
            16,   // cookie length
            0x01, 0x12, 0x23, 0x34, 0x45, 0x56, 0x67, 0x78, // cookie
            0x89, 0x9a, 0xab, 0xbc, 0xcd, 0xde, 0xef, 0xf0, // cookie
        ],
    );
}

#[test]
fn value_service_id() {
    let v = Value::ServiceId(ServiceId {
        object_uuid: uuid!("00112233-4455-6677-8899-aabbccddeeff"),
        object_cookie: uuid!("01122334-4556-6778-899a-abbccddeeff0"),
        service_uuid: uuid!("02132435-4657-6879-8a9b-acbdcedfe0f1"),
        service_cookie: uuid!("03142536-4758-697a-8b9c-adbecfd0e1f2"),
    });
    test_value_le(
        &v,
        &[
            15, // ServiceId
            16, // object_uuid length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // object_uuid
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // object_uuid
            16,   // object_cookie length
            0x01, 0x12, 0x23, 0x34, 0x45, 0x56, 0x67, 0x78, // object_cookie
            0x89, 0x9a, 0xab, 0xbc, 0xcd, 0xde, 0xef, 0xf0, // object_cookie
            16,   // service_uuid length
            0x02, 0x13, 0x24, 0x35, 0x46, 0x57, 0x68, 0x79, // service_uuid
            0x8a, 0x9b, 0xac, 0xbd, 0xce, 0xdf, 0xe0, 0xf1, // service_uuid
            16,   // service_cookie length
            0x03, 0x14, 0x25, 0x36, 0x47, 0x58, 0x69, 0x7a, // service_cookie
            0x8b, 0x9c, 0xad, 0xbe, 0xcf, 0xd0, 0xe1, 0xf2, // service_cookie
        ],
    );
    test_value_be(
        &v,
        &[
            15, // ServiceId
            16, // object_uuid length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // object_uuid
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // object_uuid
            16,   // object_cookie length
            0x01, 0x12, 0x23, 0x34, 0x45, 0x56, 0x67, 0x78, // object_cookie
            0x89, 0x9a, 0xab, 0xbc, 0xcd, 0xde, 0xef, 0xf0, // object_cookie
            16,   // service_uuid length
            0x02, 0x13, 0x24, 0x35, 0x46, 0x57, 0x68, 0x79, // service_uuid
            0x8a, 0x9b, 0xac, 0xbd, 0xce, 0xdf, 0xe0, 0xf1, // service_uuid
            16,   // service_cookie length
            0x03, 0x14, 0x25, 0x36, 0x47, 0x58, 0x69, 0x7a, // service_cookie
            0x8b, 0x9c, 0xad, 0xbe, 0xcf, 0xd0, 0xe1, 0xf2, // service_cookie
        ],
    );
}

#[test]
fn value_vec() {
    let v = Value::Vec(vec![]);
    test_value_le(
        &v,
        &[
            16, // Vec
            0,  // length
        ],
    );
    test_value_be(
        &v,
        &[
            16, // Vec
            0,  // length
        ],
    );
    let v = Value::Vec(vec![Value::None]);
    test_value_le(
        &v,
        &[
            16, // Vec
            1,  // length
            0,  // value 0
        ],
    );
    test_value_be(
        &v,
        &[
            16, // Vec
            1,  // length
            0,  // value 0
        ],
    );
}

#[test]
fn value_bytes() {
    let v = Value::Bytes(vec![]);
    test_value_le(
        &v,
        &[
            17, // Bytes
            0,  // length
        ],
    );
    test_value_be(
        &v,
        &[
            17, // Bytes
            0,  // length
        ],
    );
    let v = Value::Bytes(vec![0x12, 0x34, 0x56, 0x78]);
    test_value_le(
        &v,
        &[
            17, // Bytes
            4,  // length
            0x12, 0x34, 0x56, 0x78, // bytes
        ],
    );
    test_value_be(
        &v,
        &[
            17, // Bytes
            4,  // length
            0x12, 0x34, 0x56, 0x78, // bytes
        ],
    );
}

#[test]
fn value_u8_map() {
    let v = Value::U8Map(hashmap! {});
    test_value_le(
        &v,
        &[
            18, // U8Map
            0,  // length
        ],
    );
    test_value_be(
        &v,
        &[
            18, // U8Map
            0,  // length
        ],
    );
    let v = Value::U8Map(hashmap! { 0x12 => Value::None });
    test_value_le(
        &v,
        &[
            18,   // U8Map
            1,    // length
            0x12, // key 0
            0,    // value 0
        ],
    );
    test_value_be(
        &v,
        &[
            18,   // U8Map
            1,    // length
            0x12, // key 0
            0,    // value 0
        ],
    );
}

#[test]
fn value_i8_map() {
    let v = Value::I8Map(hashmap! {});
    test_value_le(
        &v,
        &[
            19, // I8Map
            0,  // length
        ],
    );
    test_value_be(
        &v,
        &[
            19, // I8Map
            0,  // length
        ],
    );
    let v = Value::I8Map(hashmap! { 0x12 => Value::None });
    test_value_le(
        &v,
        &[
            19,   // I8Map
            1,    // length
            0x12, // key 0
            0,    // value 0
        ],
    );
    test_value_be(
        &v,
        &[
            19,   // I8Map
            1,    // length
            0x12, // key 0
            0,    // value 0
        ],
    );
}

#[test]
fn value_u16_map() {
    let v = Value::U16Map(hashmap! {});
    test_value_le(
        &v,
        &[
            20, // U16Map
            0,  // length
        ],
    );
    test_value_be(
        &v,
        &[
            20, // U16Map
            0,  // length
        ],
    );
    let v = Value::U16Map(hashmap! { 0x1234 => Value::None });
    test_value_le(
        &v,
        &[
            20, // U16Map
            1,  // length
            251, 0x34, 0x12, // key 0
            0,    // value 0
        ],
    );
    test_value_be(
        &v,
        &[
            20, // U16Map
            1,  // length
            251, 0x12, 0x34, // key 0
            0,    // value 0
        ],
    );
}

#[test]
fn value_i16_map() {
    let v = Value::I16Map(hashmap! {});
    test_value_le(
        &v,
        &[
            21, // I16Map
            0,  // length
        ],
    );
    test_value_be(
        &v,
        &[
            21, // I16Map
            0,  // length
        ],
    );
    let v = Value::I16Map(hashmap! { 0x1234 => Value::None });
    test_value_le(
        &v,
        &[
            21, // I16Map
            1,  // length
            251, 0x68, 0x24, // key 0
            0,    // value 0
        ],
    );
    test_value_be(
        &v,
        &[
            21, // I16Map
            1,  // length
            251, 0x24, 0x68, // key 0
            0,    // value 0
        ],
    );
}

#[test]
fn value_u32_map() {
    let v = Value::U32Map(hashmap! {});
    test_value_le(
        &v,
        &[
            22, // U32Map
            0,  // length
        ],
    );
    test_value_be(
        &v,
        &[
            22, // U32Map
            0,  // length
        ],
    );
    let v = Value::U32Map(hashmap! { 0x12345678 => Value::None });
    test_value_le(
        &v,
        &[
            22, // U32Map
            1,  // length
            252, 0x78, 0x56, 0x34, 0x12, // key 0
            0,    // value 0
        ],
    );
    test_value_be(
        &v,
        &[
            22, // U32Map
            1,  // length
            252, 0x12, 0x34, 0x56, 0x78, // key 0
            0,    // value 0
        ],
    );
}

#[test]
fn value_i32_map() {
    let v = Value::I32Map(hashmap! {});
    test_value_le(
        &v,
        &[
            23, // I32Map
            0,  // length
        ],
    );
    test_value_be(
        &v,
        &[
            23, // I32Map
            0,  // length
        ],
    );
    let v = Value::I32Map(hashmap! { 0x12345678 => Value::None });
    test_value_le(
        &v,
        &[
            23, // I32Map
            1,  // length
            252, 0xf0, 0xac, 0x68, 0x24, // key 0
            0,    // value 0
        ],
    );
    test_value_be(
        &v,
        &[
            23, // I32Map
            1,  // length
            252, 0x24, 0x68, 0xac, 0xf0, // key 0
            0,    // value 0
        ],
    );
}

#[test]
fn value_u64_map() {
    let v = Value::U64Map(hashmap! {});
    test_value_le(
        &v,
        &[
            24, // U64Map
            0,  // length
        ],
    );
    test_value_be(
        &v,
        &[
            24, // U64Map
            0,  // length
        ],
    );
    let v = Value::U64Map(hashmap! { 0x123456789abcdef0 => Value::None });
    test_value_le(
        &v,
        &[
            24, // U64Map
            1,  // length
            253, 0xf0, 0xde, 0xbc, 0x9a, 0x78, 0x56, 0x34, 0x12, // key 0
            0,    // value 0
        ],
    );
    test_value_be(
        &v,
        &[
            24, // U64Map
            1,  // length
            253, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, // key 0
            0,    // value 0
        ],
    );
}

#[test]
fn value_i64_map() {
    let v = Value::I64Map(hashmap! {});
    test_value_le(
        &v,
        &[
            25, // I64Map
            0,  // length
        ],
    );
    test_value_be(
        &v,
        &[
            25, // I64Map
            0,  // length
        ],
    );
    let v = Value::I64Map(hashmap! { 0x123456789abcdef0 => Value::None });
    test_value_le(
        &v,
        &[
            25, // I64Map
            1,  // length
            253, 0xe0, 0xbd, 0x79, 0x35, 0xf1, 0xac, 0x68, 0x24, // key 0
            0,    // value 0
        ],
    );
    test_value_be(
        &v,
        &[
            25, // I64Map
            1,  // length
            253, 0x24, 0x68, 0xac, 0xf1, 0x35, 0x79, 0xbd, 0xe0, // key 0
            0,    // value 0
        ],
    );
}

#[test]
fn value_string_map() {
    let v = Value::StringMap(hashmap! {});
    test_value_le(
        &v,
        &[
            26, // StringMap
            0,  // length
        ],
    );
    test_value_be(
        &v,
        &[
            26, // StringMap
            0,  // length
        ],
    );
    let v = Value::StringMap(hashmap! { "aldrin".to_owned() => Value::None });
    test_value_le(
        &v,
        &[
            26, // StringMap
            1,  // length
            6,  // length key 0
            b'a', b'l', b'd', b'r', b'i', b'n', // value key 0
            0,    // value 0
        ],
    );
    test_value_be(
        &v,
        &[
            26, // StringMap
            1,  // length
            6,  // length key 0
            b'a', b'l', b'd', b'r', b'i', b'n', // value key 0
            0,    // value 0
        ],
    );
}

#[test]
fn value_uuid_map() {
    let v = Value::UuidMap(hashmap! {});
    test_value_le(
        &v,
        &[
            27, // UuidMap
            0,  // length
        ],
    );
    test_value_be(
        &v,
        &[
            27, // UuidMap
            0,  // length
        ],
    );
    let v =
        Value::UuidMap(hashmap! { uuid!("00112233-4455-6677-8899-aabbccddeeff") => Value::None });
    test_value_le(
        &v,
        &[
            27, // UuidMap
            1,  // length
            16, // uuid length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // key 0
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // key 0
            0,    // value 0
        ],
    );
    test_value_be(
        &v,
        &[
            27, // UuidMap
            1,  // length
            16, // uuid length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // key 0
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // key 0
            0,    // value 0
        ],
    );
}

#[test]
fn value_u8_set() {
    let v = Value::U8Set(hashset! {});
    test_value_le(
        &v,
        &[
            28, // U8Set
            0,  // length
        ],
    );
    test_value_be(
        &v,
        &[
            28, // U8Set
            0,  // length
        ],
    );
    let v = Value::U8Set(hashset! { 0x12 });
    test_value_le(
        &v,
        &[
            28,   // U8Set
            1,    // length
            0x12, // value 0
        ],
    );
    test_value_be(
        &v,
        &[
            28,   // U8Set
            1,    // length
            0x12, // value 0
        ],
    );
}

#[test]
fn value_i8_set() {
    let v = Value::I8Set(hashset! {});
    test_value_le(
        &v,
        &[
            29, // I8Set
            0,  // length
        ],
    );
    test_value_be(
        &v,
        &[
            29, // I8Set
            0,  // length
        ],
    );
    let v = Value::I8Set(hashset! { 0x12 });
    test_value_le(
        &v,
        &[
            29,   // I8Set
            1,    // length
            0x12, // value 0
        ],
    );
    test_value_be(
        &v,
        &[
            29,   // I8Set
            1,    // length
            0x12, // value 0
        ],
    );
}

#[test]
fn value_u16_set() {
    let v = Value::U16Set(hashset! {});
    test_value_le(
        &v,
        &[
            30, // U16Set
            0,  // length
        ],
    );
    test_value_be(
        &v,
        &[
            30, // U16Set
            0,  // length
        ],
    );
    let v = Value::U16Set(hashset! { 0x1234 });
    test_value_le(
        &v,
        &[
            30, // U16Set
            1,  // length
            251, 0x34, 0x12, // value 0
        ],
    );
    test_value_be(
        &v,
        &[
            30, // U16Set
            1,  // length
            251, 0x12, 0x34, // value 0
        ],
    );
}

#[test]
fn value_i16_set() {
    let v = Value::I16Set(hashset! {});
    test_value_le(
        &v,
        &[
            31, // I16Set
            0,  // length
        ],
    );
    test_value_be(
        &v,
        &[
            31, // I16Set
            0,  // length
        ],
    );
    let v = Value::I16Set(hashset! { 0x1234 });
    test_value_le(
        &v,
        &[
            31, // I16Set
            1,  // length
            251, 0x68, 0x24, // value 0
        ],
    );
    test_value_be(
        &v,
        &[
            31, // I16Set
            1,  // length
            251, 0x24, 0x68, // value 0
        ],
    );
}

#[test]
fn value_u32_set() {
    let v = Value::U32Set(hashset! {});
    test_value_le(
        &v,
        &[
            32, // U32Set
            0,  // length
        ],
    );
    test_value_be(
        &v,
        &[
            32, // U32Set
            0,  // length
        ],
    );
    let v = Value::U32Set(hashset! { 0x12345678 });
    test_value_le(
        &v,
        &[
            32, // U32Set
            1,  // length
            252, 0x78, 0x56, 0x34, 0x12, // value 0
        ],
    );
    test_value_be(
        &v,
        &[
            32, // U32Set
            1,  // length
            252, 0x12, 0x34, 0x56, 0x78, // value 0
        ],
    );
}

#[test]
fn value_i32_set() {
    let v = Value::I32Set(hashset! {});
    test_value_le(
        &v,
        &[
            33, // I32Set
            0,  // length
        ],
    );
    test_value_be(
        &v,
        &[
            33, // I32Set
            0,  // length
        ],
    );
    let v = Value::I32Set(hashset! { 0x12345678 });
    test_value_le(
        &v,
        &[
            33, // I32Set
            1,  // length
            252, 0xf0, 0xac, 0x68, 0x24, // value 0
        ],
    );
    test_value_be(
        &v,
        &[
            33, // I32Set
            1,  // length
            252, 0x24, 0x68, 0xac, 0xf0, // value 0
        ],
    );
}

#[test]
fn value_u64_set() {
    let v = Value::U64Set(hashset! {});
    test_value_le(
        &v,
        &[
            34, // U64Set
            0,  // length
        ],
    );
    test_value_be(
        &v,
        &[
            34, // U64Set
            0,  // length
        ],
    );
    let v = Value::U64Set(hashset! { 0x123456789abcdef0 });
    test_value_le(
        &v,
        &[
            34, // U64Set
            1,  // length
            253, 0xf0, 0xde, 0xbc, 0x9a, 0x78, 0x56, 0x34, 0x12, // value 0
        ],
    );
    test_value_be(
        &v,
        &[
            34, // U64Set
            1,  // length
            253, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, // value 0
        ],
    );
}

#[test]
fn value_i64_set() {
    let v = Value::I64Set(hashset! {});
    test_value_le(
        &v,
        &[
            35, // I64Set
            0,  // length
        ],
    );
    test_value_be(
        &v,
        &[
            35, // I64Set
            0,  // length
        ],
    );
    let v = Value::I64Set(hashset! { 0x123456789abcdef0 });
    test_value_le(
        &v,
        &[
            35, // I64Set
            1,  // length
            253, 0xe0, 0xbd, 0x79, 0x35, 0xf1, 0xac, 0x68, 0x24, // value 0
        ],
    );
    test_value_be(
        &v,
        &[
            35, // I64Set
            1,  // length
            253, 0x24, 0x68, 0xac, 0xf1, 0x35, 0x79, 0xbd, 0xe0, // value 0
        ],
    );
}

#[test]
fn value_string_set() {
    let v = Value::StringSet(hashset! {});
    test_value_le(
        &v,
        &[
            36, // StringSet
            0,  // length
        ],
    );
    test_value_be(
        &v,
        &[
            36, // StringSet
            0,  // length
        ],
    );
    let v = Value::StringSet(hashset! { "aldrin".to_owned() });
    test_value_le(
        &v,
        &[
            36, // StringSet
            1,  // length
            6,  // length value 0
            b'a', b'l', b'd', b'r', b'i', b'n', // value key 0
        ],
    );
    test_value_be(
        &v,
        &[
            36, // StringSet
            1,  // length
            6,  // length value 0
            b'a', b'l', b'd', b'r', b'i', b'n', // value key 0
        ],
    );
}

#[test]
fn value_uuid_set() {
    let v = Value::UuidSet(hashset! {});
    test_value_le(
        &v,
        &[
            37, // UuidSet
            0,  // length
        ],
    );
    test_value_be(
        &v,
        &[
            37, // UuidSet
            0,  // length
        ],
    );
    let v = Value::UuidSet(hashset! { uuid!("00112233-4455-6677-8899-aabbccddeeff") });
    test_value_le(
        &v,
        &[
            37, // UuidSet
            1,  // length
            16, // uuid length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // key 0
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // key 0
        ],
    );
    test_value_be(
        &v,
        &[
            37, // UuidSet
            1,  // length
            16, // uuid length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // key 0
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // key 0
        ],
    );
}

#[test]
fn value_struct() {
    let v = Value::Struct(hashmap! {});
    test_value_le(
        &v,
        &[
            38, // Struct
            0,  // length
        ],
    );
    test_value_be(
        &v,
        &[
            38, // Struct
            0,  // length
        ],
    );
    let v = Value::Struct(hashmap! { 0x12345678 => Value::None });
    test_value_le(
        &v,
        &[
            38, // Struct
            1,  // length
            252, 0x78, 0x56, 0x34, 0x12, // key 0
            0,    // value 0
        ],
    );
    test_value_be(
        &v,
        &[
            38, // Struct
            1,  // length
            252, 0x12, 0x34, 0x56, 0x78, // key 0
            0,    // value 0
        ],
    );
}

#[test]
fn value_enum() {
    let v = Value::Enum {
        variant: 0x12345678,
        value: Box::new(Value::None),
    };
    test_value_le(
        &v,
        &[
            39, // Enum
            252, 0x78, 0x56, 0x34, 0x12, // variant
            0,    // value
        ],
    );
    test_value_be(
        &v,
        &[
            39, // Enum
            252, 0x12, 0x34, 0x56, 0x78, // variant
            0,    // value
        ],
    );
}
