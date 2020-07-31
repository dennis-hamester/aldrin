use super::{BincodeSerializer, Endian, Serializer};
use aldrin_proto::*;
use bytes::{Buf, BytesMut};
use maplit::{hashmap, hashset};
use uuid::Uuid;

fn test_message(m: &Message, b: &[u8], endian: Endian) {
    let mut ser = BincodeSerializer::with_endian(endian);
    let mut b2 = BytesMut::new();

    ser.serialize(m.clone(), &mut b2).unwrap();
    assert_eq!(b, b2.bytes());

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
        22, 0, 0, 0, // CallFunctionReply
        0x78, 0x56, 0x34, 0x12, // serial
        0, 0, 0, 0, // Ok
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
        0, 0, 0, 22, // CallFunctionReply
        0x12, 0x34, 0x56, 0x78, // serial
        0, 0, 0, 0, // Ok
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
            0, 0, 0, 0, // Connect
            0x78, 0x56, 0x34, 0x12, // version
        ],
    );
    test_message_be(
        &m,
        &[
            0, 0, 0, 0, // Connect
            0x12, 0x34, 0x56, 0x78, // version
        ],
    );
}

#[test]
fn message_connect_reply() {
    let m = Message::ConnectReply(ConnectReply::Ok);
    test_message_le(
        &m,
        &[
            1, 0, 0, 0, // ConnectReply
            0, 0, 0, 0, // Ok
        ],
    );
    test_message_be(
        &m,
        &[
            0, 0, 0, 1, // ConnectReply
            0, 0, 0, 0, // Ok
        ],
    );
    let m = Message::ConnectReply(ConnectReply::VersionMismatch(0x12345678));
    test_message_le(
        &m,
        &[
            1, 0, 0, 0, // ConnectReply
            1, 0, 0, 0, // VersionMismatch
            0x78, 0x56, 0x34, 0x12, // version
        ],
    );
    test_message_be(
        &m,
        &[
            0, 0, 0, 1, // ConnectReply
            0, 0, 0, 1, // VersionMismatch
            0x12, 0x34, 0x56, 0x78, // version
        ],
    );
}

#[test]
fn message_shutdown() {
    let m = Message::Shutdown(());
    test_message_le(
        &m,
        &[
            2, 0, 0, 0, // Shutdown
        ],
    );
    test_message_be(
        &m,
        &[
            0, 0, 0, 2, // Shutdown
        ],
    );
}

#[test]
fn message_create_object() {
    let m = Message::CreateObject(CreateObject {
        serial: 0x12345678,
        uuid: Uuid::from_u128(0x00112233445566778899aabbccddeeff),
    });
    test_message_le(
        &m,
        &[
            3, 0, 0, 0, // CreateObject
            0x78, 0x56, 0x34, 0x12, // serial
            16, 0, 0, 0, 0, 0, 0, 0, // uuid length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // uuid
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // uuid
        ],
    );
    test_message_be(
        &m,
        &[
            0, 0, 0, 3, // CreateObject
            0x12, 0x34, 0x56, 0x78, // serial
            0, 0, 0, 0, 0, 0, 0, 16, // uuid length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // uuid
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // uuid
        ],
    );
}

#[test]
fn message_create_object_reply() {
    let m = Message::CreateObjectReply(CreateObjectReply {
        serial: 0x12345678,
        result: CreateObjectResult::Ok(Uuid::from_u128(0x00112233445566778899aabbccddeeff)),
    });
    test_message_le(
        &m,
        &[
            4, 0, 0, 0, // CreateObjectReply
            0x78, 0x56, 0x34, 0x12, // serial
            0, 0, 0, 0, // Ok
            16, 0, 0, 0, 0, 0, 0, 0, // uuid length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // uuid
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // uuid
        ],
    );
    test_message_be(
        &m,
        &[
            0, 0, 0, 4, // CreateObjectReply
            0x12, 0x34, 0x56, 0x78, // serial
            0, 0, 0, 0, // Ok
            0, 0, 0, 0, 0, 0, 0, 16, // uuid length
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
            4, 0, 0, 0, // CreateObjectReply
            0x78, 0x56, 0x34, 0x12, // serial
            1, 0, 0, 0, // DuplicateObject
        ],
    );
    test_message_be(
        &m,
        &[
            0, 0, 0, 4, // CreateObjectReply
            0x12, 0x34, 0x56, 0x78, // serial
            0, 0, 0, 1, // DuplicateObject
        ],
    );
}

#[test]
fn message_destroy_object() {
    let m = Message::DestroyObject(DestroyObject {
        serial: 0x12345678,
        cookie: Uuid::from_u128(0x00112233445566778899aabbccddeeff),
    });
    test_message_le(
        &m,
        &[
            5, 0, 0, 0, // DestroyObject
            0x78, 0x56, 0x34, 0x12, // serial
            16, 0, 0, 0, 0, 0, 0, 0, // cookie length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // cookie
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // cookie
        ],
    );
    test_message_be(
        &m,
        &[
            0, 0, 0, 5, // DestroyObject
            0x12, 0x34, 0x56, 0x78, // serial
            0, 0, 0, 0, 0, 0, 0, 16, // cookie length
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
            6, 0, 0, 0, // DestroyObjectReply
            0x78, 0x56, 0x34, 0x12, // serial
            0, 0, 0, 0, // Ok
        ],
    );
    test_message_be(
        &m,
        &[
            0, 0, 0, 6, // DestroyObjectReply
            0x12, 0x34, 0x56, 0x78, // serial
            0, 0, 0, 0, // Ok
        ],
    );
    let m = Message::DestroyObjectReply(DestroyObjectReply {
        serial: 0x12345678,
        result: DestroyObjectResult::InvalidObject,
    });
    test_message_le(
        &m,
        &[
            6, 0, 0, 0, // DestroyObjectReply
            0x78, 0x56, 0x34, 0x12, // serial
            1, 0, 0, 0, // InvalidObject
        ],
    );
    test_message_be(
        &m,
        &[
            0, 0, 0, 6, // DestroyObjectReply
            0x12, 0x34, 0x56, 0x78, // serial
            0, 0, 0, 1, // InvalidObject
        ],
    );
    let m = Message::DestroyObjectReply(DestroyObjectReply {
        serial: 0x12345678,
        result: DestroyObjectResult::ForeignObject,
    });
    test_message_le(
        &m,
        &[
            6, 0, 0, 0, // DestroyObjectReply
            0x78, 0x56, 0x34, 0x12, // serial
            2, 0, 0, 0, // ForeignObject
        ],
    );
    test_message_be(
        &m,
        &[
            0, 0, 0, 6, // DestroyObjectReply
            0x12, 0x34, 0x56, 0x78, // serial
            0, 0, 0, 2, // ForeignObject
        ],
    );
}

#[test]
fn message_subscribe_objects() {
    let m = Message::SubscribeObjects(SubscribeObjects { serial: None });
    test_message_le(
        &m,
        &[
            7, 0, 0, 0, // SubscribeObjects
            0, // None
        ],
    );
    test_message_be(
        &m,
        &[
            0, 0, 0, 7, // SubscribeObjects
            0, // None
        ],
    );
    let m = Message::SubscribeObjects(SubscribeObjects {
        serial: Some(0x12345678),
    });
    test_message_le(
        &m,
        &[
            7, 0, 0, 0, // SubscribeObjects
            1, // Some
            0x78, 0x56, 0x34, 0x12, // serial
        ],
    );
    test_message_be(
        &m,
        &[
            0, 0, 0, 7, // SubscribeObjects
            1, // Some
            0x12, 0x34, 0x56, 0x78, // serial
        ],
    );
}

#[test]
fn message_subscribe_objects_reply() {
    let m = Message::SubscribeObjectsReply(SubscribeObjectsReply { serial: 0x12345678 });
    test_message_le(
        &m,
        &[
            8, 0, 0, 0, // SubscribeObjectsReply
            0x78, 0x56, 0x34, 0x12, // serial
        ],
    );
    test_message_be(
        &m,
        &[
            0, 0, 0, 8, // SubscribeObjectsReply
            0x12, 0x34, 0x56, 0x78, // serial
        ],
    );
}

#[test]
fn message_unsubscribe_objects() {
    let m = Message::UnsubscribeObjects(());
    test_message_le(
        &m,
        &[
            9, 0, 0, 0, // UnsubscribeObjects
        ],
    );
    test_message_be(
        &m,
        &[
            0, 0, 0, 9, // UnsubscribeObjects
        ],
    );
}

#[test]
fn message_object_created_event() {
    let m = Message::ObjectCreatedEvent(ObjectCreatedEvent {
        uuid: Uuid::from_u128(0x00112233445566778899aabbccddeeff),
        cookie: Uuid::from_u128(0x0112233445566778899aabbccddeeff0),
        serial: None,
    });
    test_message_le(
        &m,
        &[
            10, 0, 0, 0, // ObjectCreatedEvent
            16, 0, 0, 0, 0, 0, 0, 0, // uuid length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // uuid
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // uuid
            16, 0, 0, 0, 0, 0, 0, 0, // cookie length
            0x01, 0x12, 0x23, 0x34, 0x45, 0x56, 0x67, 0x78, // cookie
            0x89, 0x9a, 0xab, 0xbc, 0xcd, 0xde, 0xef, 0xf0, // cookie
            0,    // None
        ],
    );
    test_message_be(
        &m,
        &[
            0, 0, 0, 10, // ObjectCreatedEvent
            0, 0, 0, 0, 0, 0, 0, 16, // uuid length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // uuid
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // uuid
            0, 0, 0, 0, 0, 0, 0, 16, // cookie length
            0x01, 0x12, 0x23, 0x34, 0x45, 0x56, 0x67, 0x78, // cookie
            0x89, 0x9a, 0xab, 0xbc, 0xcd, 0xde, 0xef, 0xf0, // cookie
            0,    // None
        ],
    );
    let m = Message::ObjectCreatedEvent(ObjectCreatedEvent {
        uuid: Uuid::from_u128(0x00112233445566778899aabbccddeeff),
        cookie: Uuid::from_u128(0x0112233445566778899aabbccddeeff0),
        serial: Some(0x12345678),
    });
    test_message_le(
        &m,
        &[
            10, 0, 0, 0, // ObjectCreatedEvent
            16, 0, 0, 0, 0, 0, 0, 0, // uuid length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // uuid
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // uuid
            16, 0, 0, 0, 0, 0, 0, 0, // cookie length
            0x01, 0x12, 0x23, 0x34, 0x45, 0x56, 0x67, 0x78, // cookie
            0x89, 0x9a, 0xab, 0xbc, 0xcd, 0xde, 0xef, 0xf0, // cookie
            1,    // Some
            0x78, 0x56, 0x34, 0x12, // serial
        ],
    );
    test_message_be(
        &m,
        &[
            0, 0, 0, 10, // ObjectCreatedEvent
            0, 0, 0, 0, 0, 0, 0, 16, // uuid length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // uuid
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // uuid
            0, 0, 0, 0, 0, 0, 0, 16, // cookie length
            0x01, 0x12, 0x23, 0x34, 0x45, 0x56, 0x67, 0x78, // cookie
            0x89, 0x9a, 0xab, 0xbc, 0xcd, 0xde, 0xef, 0xf0, // cookie
            1,    // Some
            0x12, 0x34, 0x56, 0x78, // serial
        ],
    );
}

#[test]
fn message_object_destroyed_event() {
    let m = Message::ObjectDestroyedEvent(ObjectDestroyedEvent {
        uuid: Uuid::from_u128(0x00112233445566778899aabbccddeeff),
        cookie: Uuid::from_u128(0x0112233445566778899aabbccddeeff0),
    });
    test_message_le(
        &m,
        &[
            11, 0, 0, 0, // ObjectDestroyedEvent
            16, 0, 0, 0, 0, 0, 0, 0, // uuid length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // uuid
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // uuid
            16, 0, 0, 0, 0, 0, 0, 0, // cookie length
            0x01, 0x12, 0x23, 0x34, 0x45, 0x56, 0x67, 0x78, // cookie
            0x89, 0x9a, 0xab, 0xbc, 0xcd, 0xde, 0xef, 0xf0, // cookie
        ],
    );
    test_message_be(
        &m,
        &[
            0, 0, 0, 11, // ObjectDestroyedEvent
            0, 0, 0, 0, 0, 0, 0, 16, // uuid length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // uuid
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // uuid
            0, 0, 0, 0, 0, 0, 0, 16, // cookie length
            0x01, 0x12, 0x23, 0x34, 0x45, 0x56, 0x67, 0x78, // cookie
            0x89, 0x9a, 0xab, 0xbc, 0xcd, 0xde, 0xef, 0xf0, // cookie
        ],
    );
}

#[test]
fn message_create_service() {
    let m = Message::CreateService(CreateService {
        serial: 0x12345678,
        object_cookie: Uuid::from_u128(0x00112233445566778899aabbccddeeff),
        uuid: Uuid::from_u128(0x0112233445566778899aabbccddeeff0),
    });
    test_message_le(
        &m,
        &[
            12, 0, 0, 0, // CreateService
            0x78, 0x56, 0x34, 0x12, // serial
            16, 0, 0, 0, 0, 0, 0, 0, // object_cookie length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // object_cookie
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // object_cookie
            16, 0, 0, 0, 0, 0, 0, 0, // uuid length
            0x01, 0x12, 0x23, 0x34, 0x45, 0x56, 0x67, 0x78, // uuid
            0x89, 0x9a, 0xab, 0xbc, 0xcd, 0xde, 0xef, 0xf0, // uuid
        ],
    );
    test_message_be(
        &m,
        &[
            0, 0, 0, 12, // CreateService
            0x12, 0x34, 0x56, 0x78, // serial
            0, 0, 0, 0, 0, 0, 0, 16, // object_cookie length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // object_cookie
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // object_cookie
            0, 0, 0, 0, 0, 0, 0, 16, // uuid length
            0x01, 0x12, 0x23, 0x34, 0x45, 0x56, 0x67, 0x78, // uuid
            0x89, 0x9a, 0xab, 0xbc, 0xcd, 0xde, 0xef, 0xf0, // uuid
        ],
    );
}

#[test]
fn message_create_service_reply() {
    let m = Message::CreateServiceReply(CreateServiceReply {
        serial: 0x12345678,
        result: CreateServiceResult::Ok(Uuid::from_u128(0x00112233445566778899aabbccddeeff)),
    });
    test_message_le(
        &m,
        &[
            13, 0, 0, 0, // CreateServiceReply
            0x78, 0x56, 0x34, 0x12, // serial
            0, 0, 0, 0, // Ok
            16, 0, 0, 0, 0, 0, 0, 0, // cookie length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // cookie
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // cookie
        ],
    );
    test_message_be(
        &m,
        &[
            0, 0, 0, 13, // CreateServiceReply
            0x12, 0x34, 0x56, 0x78, // serial
            0, 0, 0, 0, // Ok
            0, 0, 0, 0, 0, 0, 0, 16, // cookie length
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
            13, 0, 0, 0, // CreateServiceReply
            0x78, 0x56, 0x34, 0x12, // serial
            1, 0, 0, 0, // DuplicateService
        ],
    );
    test_message_be(
        &m,
        &[
            0, 0, 0, 13, // CreateServiceReply
            0x12, 0x34, 0x56, 0x78, // serial
            0, 0, 0, 1, // DuplicateService
        ],
    );
    let m = Message::CreateServiceReply(CreateServiceReply {
        serial: 0x12345678,
        result: CreateServiceResult::InvalidObject,
    });
    test_message_le(
        &m,
        &[
            13, 0, 0, 0, // CreateServiceReply
            0x78, 0x56, 0x34, 0x12, // serial
            2, 0, 0, 0, // InvalidObject
        ],
    );
    test_message_be(
        &m,
        &[
            0, 0, 0, 13, // CreateServiceReply
            0x12, 0x34, 0x56, 0x78, // serial
            0, 0, 0, 2, // InvalidObject
        ],
    );
    let m = Message::CreateServiceReply(CreateServiceReply {
        serial: 0x12345678,
        result: CreateServiceResult::ForeignObject,
    });
    test_message_le(
        &m,
        &[
            13, 0, 0, 0, // CreateServiceReply
            0x78, 0x56, 0x34, 0x12, // serial
            3, 0, 0, 0, // ForeignObject
        ],
    );
    test_message_be(
        &m,
        &[
            0, 0, 0, 13, // CreateServiceReply
            0x12, 0x34, 0x56, 0x78, // serial
            0, 0, 0, 3, // ForeignObject
        ],
    );
}

#[test]
fn message_destroy_service() {
    let m = Message::DestroyService(DestroyService {
        serial: 0x12345678,
        cookie: Uuid::from_u128(0x00112233445566778899aabbccddeeff),
    });
    test_message_le(
        &m,
        &[
            14, 0, 0, 0, // DestroyService
            0x78, 0x56, 0x34, 0x12, // serial
            16, 0, 0, 0, 0, 0, 0, 0, // cookie length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // cookie
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // cookie
        ],
    );
    test_message_be(
        &m,
        &[
            0, 0, 0, 14, // DestroyService
            0x12, 0x34, 0x56, 0x78, // serial
            0, 0, 0, 0, 0, 0, 0, 16, // cookie length
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
            15, 0, 0, 0, // DestroyServiceReply
            0x78, 0x56, 0x34, 0x12, // serial
            0, 0, 0, 0, // Ok
        ],
    );
    test_message_be(
        &m,
        &[
            0, 0, 0, 15, // DestroyServiceReply
            0x12, 0x34, 0x56, 0x78, // serial
            0, 0, 0, 0, // Ok
        ],
    );
    let m = Message::DestroyServiceReply(DestroyServiceReply {
        serial: 0x12345678,
        result: DestroyServiceResult::InvalidService,
    });
    test_message_le(
        &m,
        &[
            15, 0, 0, 0, // DestroyServiceReply
            0x78, 0x56, 0x34, 0x12, // serial
            1, 0, 0, 0, // InvalidService
        ],
    );
    test_message_be(
        &m,
        &[
            0, 0, 0, 15, // DestroyServiceReply
            0x12, 0x34, 0x56, 0x78, // serial
            0, 0, 0, 1, // InvalidService
        ],
    );
    let m = Message::DestroyServiceReply(DestroyServiceReply {
        serial: 0x12345678,
        result: DestroyServiceResult::ForeignObject,
    });
    test_message_le(
        &m,
        &[
            15, 0, 0, 0, // DestroyServiceReply
            0x78, 0x56, 0x34, 0x12, // serial
            2, 0, 0, 0, // ForeignObject
        ],
    );
    test_message_be(
        &m,
        &[
            0, 0, 0, 15, // DestroyServiceReply
            0x12, 0x34, 0x56, 0x78, // serial
            0, 0, 0, 2, // ForeignObject
        ],
    );
}

#[test]
fn message_subscribe_services() {
    let m = Message::SubscribeServices(SubscribeServices { serial: None });
    test_message_le(
        &m,
        &[
            16, 0, 0, 0, // SubscribeServices
            0, // None
        ],
    );
    test_message_be(
        &m,
        &[
            0, 0, 0, 16, // SubscribeServices
            0,  // None
        ],
    );
    let m = Message::SubscribeServices(SubscribeServices {
        serial: Some(0x12345678),
    });
    test_message_le(
        &m,
        &[
            16, 0, 0, 0, // SubscribeServices
            1, // Some
            0x78, 0x56, 0x34, 0x12, // serial
        ],
    );
    test_message_be(
        &m,
        &[
            0, 0, 0, 16, // SubscribeServices
            1,  // Some
            0x12, 0x34, 0x56, 0x78, // serial
        ],
    );
}

#[test]
fn message_subscribe_services_reply() {
    let m = Message::SubscribeServicesReply(SubscribeServicesReply { serial: 0x12345678 });
    test_message_le(
        &m,
        &[
            17, 0, 0, 0, // SubscribeServicesReply
            0x78, 0x56, 0x34, 0x12, // serial
        ],
    );
    test_message_be(
        &m,
        &[
            0, 0, 0, 17, // SubscribeServicesReply
            0x12, 0x34, 0x56, 0x78, // serial
        ],
    );
}

#[test]
fn message_unsubscribe_services() {
    let m = Message::UnsubscribeServices(());
    test_message_le(
        &m,
        &[
            18, 0, 0, 0, // UnsubscribeServices
        ],
    );
    test_message_be(
        &m,
        &[
            0, 0, 0, 18, // UnsubscribeServices
        ],
    );
}

#[test]
fn message_service_created_event() {
    let m = Message::ServiceCreatedEvent(ServiceCreatedEvent {
        object_uuid: Uuid::from_u128(0x00112233445566778899aabbccddeeff),
        object_cookie: Uuid::from_u128(0x0112233445566778899aabbccddeeff0),
        uuid: Uuid::from_u128(0x02132435465768798a9bacbdcedfe0f1),
        cookie: Uuid::from_u128(0x031425364758697a8b9cadbecfd0e1f2),
        serial: None,
    });
    test_message_le(
        &m,
        &[
            19, 0, 0, 0, // ServiceCreatedEvent
            16, 0, 0, 0, 0, 0, 0, 0, // object_uuid length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // object_uuid
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // object_uuid
            16, 0, 0, 0, 0, 0, 0, 0, // object_cookie length
            0x01, 0x12, 0x23, 0x34, 0x45, 0x56, 0x67, 0x78, // object_cookie
            0x89, 0x9a, 0xab, 0xbc, 0xcd, 0xde, 0xef, 0xf0, // object_cookie
            16, 0, 0, 0, 0, 0, 0, 0, // uuid length
            0x02, 0x13, 0x24, 0x35, 0x46, 0x57, 0x68, 0x79, // uuid
            0x8a, 0x9b, 0xac, 0xbd, 0xce, 0xdf, 0xe0, 0xf1, // uuid
            16, 0, 0, 0, 0, 0, 0, 0, // cookie length
            0x03, 0x14, 0x25, 0x36, 0x47, 0x58, 0x69, 0x7a, // cookie
            0x8b, 0x9c, 0xad, 0xbe, 0xcf, 0xd0, 0xe1, 0xf2, // cookie
            0,    // None
        ],
    );
    test_message_be(
        &m,
        &[
            0, 0, 0, 19, // ServiceCreatedEvent
            0, 0, 0, 0, 0, 0, 0, 16, // object_uuid length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // object_uuid
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // object_uuid
            0, 0, 0, 0, 0, 0, 0, 16, // object_cookie length
            0x01, 0x12, 0x23, 0x34, 0x45, 0x56, 0x67, 0x78, // object_cookie
            0x89, 0x9a, 0xab, 0xbc, 0xcd, 0xde, 0xef, 0xf0, // object_cookie
            0, 0, 0, 0, 0, 0, 0, 16, // uuid length
            0x02, 0x13, 0x24, 0x35, 0x46, 0x57, 0x68, 0x79, // uuid
            0x8a, 0x9b, 0xac, 0xbd, 0xce, 0xdf, 0xe0, 0xf1, // uuid
            0, 0, 0, 0, 0, 0, 0, 16, // cookie length
            0x03, 0x14, 0x25, 0x36, 0x47, 0x58, 0x69, 0x7a, // cookie
            0x8b, 0x9c, 0xad, 0xbe, 0xcf, 0xd0, 0xe1, 0xf2, // cookie
            0,    // None
        ],
    );
    let m = Message::ServiceCreatedEvent(ServiceCreatedEvent {
        object_uuid: Uuid::from_u128(0x00112233445566778899aabbccddeeff),
        object_cookie: Uuid::from_u128(0x0112233445566778899aabbccddeeff0),
        uuid: Uuid::from_u128(0x02132435465768798a9bacbdcedfe0f1),
        cookie: Uuid::from_u128(0x031425364758697a8b9cadbecfd0e1f2),
        serial: Some(0x12345678),
    });
    test_message_le(
        &m,
        &[
            19, 0, 0, 0, // ServiceCreatedEvent
            16, 0, 0, 0, 0, 0, 0, 0, // object_uuid length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // object_uuid
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // object_uuid
            16, 0, 0, 0, 0, 0, 0, 0, // object_cookie length
            0x01, 0x12, 0x23, 0x34, 0x45, 0x56, 0x67, 0x78, // object_cookie
            0x89, 0x9a, 0xab, 0xbc, 0xcd, 0xde, 0xef, 0xf0, // object_cookie
            16, 0, 0, 0, 0, 0, 0, 0, // uuid length
            0x02, 0x13, 0x24, 0x35, 0x46, 0x57, 0x68, 0x79, // uuid
            0x8a, 0x9b, 0xac, 0xbd, 0xce, 0xdf, 0xe0, 0xf1, // uuid
            16, 0, 0, 0, 0, 0, 0, 0, // cookie length
            0x03, 0x14, 0x25, 0x36, 0x47, 0x58, 0x69, 0x7a, // cookie
            0x8b, 0x9c, 0xad, 0xbe, 0xcf, 0xd0, 0xe1, 0xf2, // cookie
            1,    // Some
            0x78, 0x56, 0x34, 0x12, // serial
        ],
    );
    test_message_be(
        &m,
        &[
            0, 0, 0, 19, // ServiceCreatedEvent
            0, 0, 0, 0, 0, 0, 0, 16, // object_uuid length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // object_uuid
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // object_uuid
            0, 0, 0, 0, 0, 0, 0, 16, // object_cookie length
            0x01, 0x12, 0x23, 0x34, 0x45, 0x56, 0x67, 0x78, // object_cookie
            0x89, 0x9a, 0xab, 0xbc, 0xcd, 0xde, 0xef, 0xf0, // object_cookie
            0, 0, 0, 0, 0, 0, 0, 16, // uuid length
            0x02, 0x13, 0x24, 0x35, 0x46, 0x57, 0x68, 0x79, // uuid
            0x8a, 0x9b, 0xac, 0xbd, 0xce, 0xdf, 0xe0, 0xf1, // uuid
            0, 0, 0, 0, 0, 0, 0, 16, // cookie length
            0x03, 0x14, 0x25, 0x36, 0x47, 0x58, 0x69, 0x7a, // cookie
            0x8b, 0x9c, 0xad, 0xbe, 0xcf, 0xd0, 0xe1, 0xf2, // cookie
            1,    // Some
            0x12, 0x34, 0x56, 0x78, // serial
        ],
    );
}

#[test]
fn message_service_destroyed_event() {
    let m = Message::ServiceDestroyedEvent(ServiceDestroyedEvent {
        object_uuid: Uuid::from_u128(0x00112233445566778899aabbccddeeff),
        object_cookie: Uuid::from_u128(0x0112233445566778899aabbccddeeff0),
        uuid: Uuid::from_u128(0x02132435465768798a9bacbdcedfe0f1),
        cookie: Uuid::from_u128(0x031425364758697a8b9cadbecfd0e1f2),
    });
    test_message_le(
        &m,
        &[
            20, 0, 0, 0, // ServiceDestroyedEvent
            16, 0, 0, 0, 0, 0, 0, 0, // object_uuid length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // object_uuid
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // object_uuid
            16, 0, 0, 0, 0, 0, 0, 0, // object_cookie length
            0x01, 0x12, 0x23, 0x34, 0x45, 0x56, 0x67, 0x78, // object_cookie
            0x89, 0x9a, 0xab, 0xbc, 0xcd, 0xde, 0xef, 0xf0, // object_cookie
            16, 0, 0, 0, 0, 0, 0, 0, // uuid length
            0x02, 0x13, 0x24, 0x35, 0x46, 0x57, 0x68, 0x79, // uuid
            0x8a, 0x9b, 0xac, 0xbd, 0xce, 0xdf, 0xe0, 0xf1, // uuid
            16, 0, 0, 0, 0, 0, 0, 0, // cookie length
            0x03, 0x14, 0x25, 0x36, 0x47, 0x58, 0x69, 0x7a, // cookie
            0x8b, 0x9c, 0xad, 0xbe, 0xcf, 0xd0, 0xe1, 0xf2, // cookie
        ],
    );
    test_message_be(
        &m,
        &[
            0, 0, 0, 20, // ServiceDestroyedEvent
            0, 0, 0, 0, 0, 0, 0, 16, // object_uuid length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // object_uuid
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // object_uuid
            0, 0, 0, 0, 0, 0, 0, 16, // object_cookie length
            0x01, 0x12, 0x23, 0x34, 0x45, 0x56, 0x67, 0x78, // object_cookie
            0x89, 0x9a, 0xab, 0xbc, 0xcd, 0xde, 0xef, 0xf0, // object_cookie
            0, 0, 0, 0, 0, 0, 0, 16, // uuid length
            0x02, 0x13, 0x24, 0x35, 0x46, 0x57, 0x68, 0x79, // uuid
            0x8a, 0x9b, 0xac, 0xbd, 0xce, 0xdf, 0xe0, 0xf1, // uuid
            0, 0, 0, 0, 0, 0, 0, 16, // cookie length
            0x03, 0x14, 0x25, 0x36, 0x47, 0x58, 0x69, 0x7a, // cookie
            0x8b, 0x9c, 0xad, 0xbe, 0xcf, 0xd0, 0xe1, 0xf2, // cookie
        ],
    );
}

#[test]
fn message_call_function() {
    let m = Message::CallFunction(CallFunction {
        serial: 0x12345678,
        service_cookie: Uuid::from_u128(0x00112233445566778899aabbccddeeff),
        function: 0x87654321,
        args: Value::None,
    });
    test_message_le(
        &m,
        &[
            21, 0, 0, 0, // CallFunction
            0x78, 0x56, 0x34, 0x12, // serial
            16, 0, 0, 0, 0, 0, 0, 0, // service_cookie length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // service_cookie
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // service_cookie
            0x21, 0x43, 0x65, 0x87, // function
            0, 0, 0, 0, // args
        ],
    );
    test_message_be(
        &m,
        &[
            0, 0, 0, 21, // CallFunction
            0x12, 0x34, 0x56, 0x78, // serial
            0, 0, 0, 0, 0, 0, 0, 16, // service_cookie length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // service_cookie
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // service_cookie
            0x87, 0x65, 0x43, 0x21, // function
            0, 0, 0, 0, // args
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
            22, 0, 0, 0, // CallFunctionReply
            0x78, 0x56, 0x34, 0x12, // serial
            0, 0, 0, 0, // Ok
            0, 0, 0, 0, // value
        ],
    );
    test_message_be(
        &m,
        &[
            0, 0, 0, 22, // CallFunctionReply
            0x12, 0x34, 0x56, 0x78, // serial
            0, 0, 0, 0, // Ok
            0, 0, 0, 0, // value
        ],
    );
    let m = Message::CallFunctionReply(CallFunctionReply {
        serial: 0x12345678,
        result: CallFunctionResult::Err(Value::None),
    });
    test_message_le(
        &m,
        &[
            22, 0, 0, 0, // CallFunctionReply
            0x78, 0x56, 0x34, 0x12, // serial
            1, 0, 0, 0, // Err
            0, 0, 0, 0, // value
        ],
    );
    test_message_be(
        &m,
        &[
            0, 0, 0, 22, // CallFunctionReply
            0x12, 0x34, 0x56, 0x78, // serial
            0, 0, 0, 1, // Err
            0, 0, 0, 0, // value
        ],
    );
    let m = Message::CallFunctionReply(CallFunctionReply {
        serial: 0x12345678,
        result: CallFunctionResult::Aborted,
    });
    test_message_le(
        &m,
        &[
            22, 0, 0, 0, // CallFunctionReply
            0x78, 0x56, 0x34, 0x12, // serial
            2, 0, 0, 0, // Aborted
        ],
    );
    test_message_be(
        &m,
        &[
            0, 0, 0, 22, // CallFunctionReply
            0x12, 0x34, 0x56, 0x78, // serial
            0, 0, 0, 2, // Aborted
        ],
    );
    let m = Message::CallFunctionReply(CallFunctionReply {
        serial: 0x12345678,
        result: CallFunctionResult::InvalidService,
    });
    test_message_le(
        &m,
        &[
            22, 0, 0, 0, // CallFunctionReply
            0x78, 0x56, 0x34, 0x12, // serial
            3, 0, 0, 0, // InvalidService
        ],
    );
    test_message_be(
        &m,
        &[
            0, 0, 0, 22, // CallFunctionReply
            0x12, 0x34, 0x56, 0x78, // serial
            0, 0, 0, 3, // InvalidService
        ],
    );
    let m = Message::CallFunctionReply(CallFunctionReply {
        serial: 0x12345678,
        result: CallFunctionResult::InvalidFunction,
    });
    test_message_le(
        &m,
        &[
            22, 0, 0, 0, // CallFunctionReply
            0x78, 0x56, 0x34, 0x12, // serial
            4, 0, 0, 0, // InvalidFunction
        ],
    );
    test_message_be(
        &m,
        &[
            0, 0, 0, 22, // CallFunctionReply
            0x12, 0x34, 0x56, 0x78, // serial
            0, 0, 0, 4, // InvalidFunction
        ],
    );
    let m = Message::CallFunctionReply(CallFunctionReply {
        serial: 0x12345678,
        result: CallFunctionResult::InvalidArgs,
    });
    test_message_le(
        &m,
        &[
            22, 0, 0, 0, // CallFunctionReply
            0x78, 0x56, 0x34, 0x12, // serial
            5, 0, 0, 0, // InvalidArgs
        ],
    );
    test_message_be(
        &m,
        &[
            0, 0, 0, 22, // CallFunctionReply
            0x12, 0x34, 0x56, 0x78, // serial
            0, 0, 0, 5, // InvalidArgs
        ],
    );
}

#[test]
fn message_subscribe_event() {
    let m = Message::SubscribeEvent(SubscribeEvent {
        serial: None,
        service_cookie: Uuid::from_u128(0x00112233445566778899aabbccddeeff),
        event: 0x87654321,
    });
    test_message_le(
        &m,
        &[
            23, 0, 0, 0, // SubscribeEvent
            0, // None
            16, 0, 0, 0, 0, 0, 0, 0, // service_cookie length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // service_cookie
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // service_cookie
            0x21, 0x43, 0x65, 0x87, // event
        ],
    );
    test_message_be(
        &m,
        &[
            0, 0, 0, 23, // SubscribeEvent
            0,  // None
            0, 0, 0, 0, 0, 0, 0, 16, // service_cookie length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // service_cookie
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // service_cookie
            0x87, 0x65, 0x43, 0x21, // event
        ],
    );
    let m = Message::SubscribeEvent(SubscribeEvent {
        serial: Some(0x12345678),
        service_cookie: Uuid::from_u128(0x00112233445566778899aabbccddeeff),
        event: 0x87654321,
    });
    test_message_le(
        &m,
        &[
            23, 0, 0, 0, // SubscribeEvent
            1, // Some
            0x78, 0x56, 0x34, 0x12, // serial
            16, 0, 0, 0, 0, 0, 0, 0, // service_cookie length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // service_cookie
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // service_cookie
            0x21, 0x43, 0x65, 0x87, // event
        ],
    );
    test_message_be(
        &m,
        &[
            0, 0, 0, 23, // SubscribeEvent
            1,  // Some
            0x12, 0x34, 0x56, 0x78, // serial
            0, 0, 0, 0, 0, 0, 0, 16, // service_cookie length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // service_cookie
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // service_cookie
            0x87, 0x65, 0x43, 0x21, // event
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
            24, 0, 0, 0, // SubscribeEventReply
            0x78, 0x56, 0x34, 0x12, // serial
            0, 0, 0, 0, // Ok
        ],
    );
    test_message_be(
        &m,
        &[
            0, 0, 0, 24, // SubscribeEventReply
            0x12, 0x34, 0x56, 0x78, // serial
            0, 0, 0, 0, // Ok
        ],
    );
    let m = Message::SubscribeEventReply(SubscribeEventReply {
        serial: 0x12345678,
        result: SubscribeEventResult::InvalidService,
    });
    test_message_le(
        &m,
        &[
            24, 0, 0, 0, // SubscribeEventReply
            0x78, 0x56, 0x34, 0x12, // serial
            1, 0, 0, 0, // InvalidService
        ],
    );
    test_message_be(
        &m,
        &[
            0, 0, 0, 24, // SubscribeEventReply
            0x12, 0x34, 0x56, 0x78, // serial
            0, 0, 0, 1, // InvalidService
        ],
    );
}

#[test]
fn message_unsubscribe_event() {
    let m = Message::UnsubscribeEvent(UnsubscribeEvent {
        service_cookie: Uuid::from_u128(0x00112233445566778899aabbccddeeff),
        event: 0x87654321,
    });
    test_message_le(
        &m,
        &[
            25, 0, 0, 0, // UnsubscribeEvent
            16, 0, 0, 0, 0, 0, 0, 0, // service_cookie length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // service_cookie
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // service_cookie
            0x21, 0x43, 0x65, 0x87, // event
        ],
    );
    test_message_be(
        &m,
        &[
            0, 0, 0, 25, // UnsubscribeEvent
            0, 0, 0, 0, 0, 0, 0, 16, // uuid length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // service_cookie
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // service_cookie
            0x87, 0x65, 0x43, 0x21, // event
        ],
    );
}

#[test]
fn message_emit_event() {
    let m = Message::EmitEvent(EmitEvent {
        service_cookie: Uuid::from_u128(0x00112233445566778899aabbccddeeff),
        event: 0x87654321,
        args: Value::None,
    });
    test_message_le(
        &m,
        &[
            26, 0, 0, 0, // EmitEvent
            16, 0, 0, 0, 0, 0, 0, 0, // service_cookie length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // service_cookie
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // service_cookie
            0x21, 0x43, 0x65, 0x87, // event
            0, 0, 0, 0, // args
        ],
    );
    test_message_be(
        &m,
        &[
            0, 0, 0, 26, // EmitEvent
            0, 0, 0, 0, 0, 0, 0, 16, // service_cookie length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // service_cookie
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // service_cookie
            0x87, 0x65, 0x43, 0x21, // event
            0, 0, 0, 0, // args
        ],
    );
}

#[test]
fn value_none() {
    let v = Value::None;
    test_value_le(
        &v,
        &[
            0, 0, 0, 0, // None
        ],
    );
    test_value_be(
        &v,
        &[
            0, 0, 0, 0, // None
        ],
    );
}

#[test]
fn value_bool() {
    let v = Value::Bool(false);
    test_value_le(
        &v,
        &[
            1, 0, 0, 0, // Bool
            0, // false
        ],
    );
    test_value_be(
        &v,
        &[
            0, 0, 0, 1, // Bool
            0, // false
        ],
    );
    let v = Value::Bool(true);
    test_value_le(
        &v,
        &[
            1, 0, 0, 0, // Bool
            1, // true
        ],
    );
    test_value_be(
        &v,
        &[
            0, 0, 0, 1, // Bool
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
            2, 0, 0, 0,    // U8
            0x12, // value
        ],
    );
    test_value_be(
        &v,
        &[
            0, 0, 0, 2,    // U8
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
            3, 0, 0, 0,    // I8
            0x12, // value
        ],
    );
    test_value_be(
        &v,
        &[
            0, 0, 0, 3,    // I8
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
            4, 0, 0, 0, // U16
            0x34, 0x12, // value
        ],
    );
    test_value_be(
        &v,
        &[
            0, 0, 0, 4, // U16
            0x12, 0x34, // value
        ],
    );
}

#[test]
fn value_i16() {
    let v = Value::I16(0x1234);
    test_value_le(
        &v,
        &[
            5, 0, 0, 0, // I16
            0x34, 0x12, // value
        ],
    );
    test_value_be(
        &v,
        &[
            0, 0, 0, 5, // I16
            0x12, 0x34, // value
        ],
    );
}

#[test]
fn value_u32() {
    let v = Value::U32(0x12345678);
    test_value_le(
        &v,
        &[
            6, 0, 0, 0, // U32
            0x78, 0x56, 0x34, 0x12, // value
        ],
    );
    test_value_be(
        &v,
        &[
            0, 0, 0, 6, // U32
            0x12, 0x34, 0x56, 0x78, // value
        ],
    );
}

#[test]
fn value_i32() {
    let v = Value::I32(0x12345678);
    test_value_le(
        &v,
        &[
            7, 0, 0, 0, // I32
            0x78, 0x56, 0x34, 0x12, // value
        ],
    );
    test_value_be(
        &v,
        &[
            0, 0, 0, 7, // I32
            0x12, 0x34, 0x56, 0x78, // value
        ],
    );
}

#[test]
fn value_u64() {
    let v = Value::U64(0x123456789abcdef0);
    test_value_le(
        &v,
        &[
            8, 0, 0, 0, // U64
            0xf0, 0xde, 0xbc, 0x9a, 0x78, 0x56, 0x34, 0x12, // value
        ],
    );
    test_value_be(
        &v,
        &[
            0, 0, 0, 8, // U64
            0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, // value
        ],
    );
}

#[test]
fn value_i64() {
    let v = Value::I64(0x123456789abcdef0);
    test_value_le(
        &v,
        &[
            9, 0, 0, 0, // I64
            0xf0, 0xde, 0xbc, 0x9a, 0x78, 0x56, 0x34, 0x12, // value
        ],
    );
    test_value_be(
        &v,
        &[
            0, 0, 0, 9, // I64
            0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, // value
        ],
    );
}

#[test]
fn value_f32() {
    let v = Value::F32(1.7378244e34);
    test_value_le(
        &v,
        &[
            10, 0, 0, 0, // F32
            0x12, 0x34, 0x56, 0x78, // value
        ],
    );
    test_value_be(
        &v,
        &[
            0, 0, 0, 10, // F32
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
            11, 0, 0, 0, // F64
            0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, // value
        ],
    );
    test_value_be(
        &v,
        &[
            0, 0, 0, 11, // F64
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
            12, 0, 0, 0, // String
            0, 0, 0, 0, 0, 0, 0, 0, // length
        ],
    );
    test_value_be(
        &v,
        &[
            0, 0, 0, 12, // String
            0, 0, 0, 0, 0, 0, 0, 0, // length
        ],
    );
    let v = Value::String("aldrin".to_owned());
    test_value_le(
        &v,
        &[
            12, 0, 0, 0, // String
            6, 0, 0, 0, 0, 0, 0, 0, // length
            b'a', b'l', b'd', b'r', b'i', b'n', // value
        ],
    );
    test_value_be(
        &v,
        &[
            0, 0, 0, 12, // String
            0, 0, 0, 0, 0, 0, 0, 6, // length
            b'a', b'l', b'd', b'r', b'i', b'n', // value
        ],
    );
}

#[test]
fn value_uuid() {
    let v = Value::Uuid(Uuid::from_u128(0x00112233445566778899aabbccddeeff));
    test_value_le(
        &v,
        &[
            13, 0, 0, 0, // Uuid
            16, 0, 0, 0, 0, 0, 0, 0, // uuid length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // value
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // value
        ],
    );
    test_value_be(
        &v,
        &[
            0, 0, 0, 13, // Uuid
            0, 0, 0, 0, 0, 0, 0, 16, // uuid length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // value
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // value
        ],
    );
}

#[test]
fn value_vec() {
    let v = Value::Vec(vec![]);
    test_value_le(
        &v,
        &[
            14, 0, 0, 0, // Vec
            0, 0, 0, 0, 0, 0, 0, 0, // length
        ],
    );
    test_value_be(
        &v,
        &[
            0, 0, 0, 14, // Vec
            0, 0, 0, 0, 0, 0, 0, 0, // length
        ],
    );
    let v = Value::Vec(vec![Value::None]);
    test_value_le(
        &v,
        &[
            14, 0, 0, 0, // Vec
            1, 0, 0, 0, 0, 0, 0, 0, // length
            0, 0, 0, 0, // value 0
        ],
    );
    test_value_be(
        &v,
        &[
            0, 0, 0, 14, // Vec
            0, 0, 0, 0, 0, 0, 0, 1, // length
            0, 0, 0, 0, // value 0
        ],
    );
}

#[test]
fn value_bytes() {
    let v = Value::Bytes(vec![]);
    test_value_le(
        &v,
        &[
            15, 0, 0, 0, // Bytes
            0, 0, 0, 0, 0, 0, 0, 0, // length
        ],
    );
    test_value_be(
        &v,
        &[
            0, 0, 0, 15, // Bytes
            0, 0, 0, 0, 0, 0, 0, 0, // length
        ],
    );
    let v = Value::Bytes(vec![0x12, 0x34, 0x56, 0x78]);
    test_value_le(
        &v,
        &[
            15, 0, 0, 0, // Bytes
            4, 0, 0, 0, 0, 0, 0, 0, // length
            0x12, 0x34, 0x56, 0x78, // bytes
        ],
    );
    test_value_be(
        &v,
        &[
            0, 0, 0, 15, // Bytes
            0, 0, 0, 0, 0, 0, 0, 4, // length
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
            16, 0, 0, 0, // U8Map
            0, 0, 0, 0, 0, 0, 0, 0, // length
        ],
    );
    test_value_be(
        &v,
        &[
            0, 0, 0, 16, // U8Map
            0, 0, 0, 0, 0, 0, 0, 0, // length
        ],
    );
    let v = Value::U8Map(hashmap! { 0x12 => Value::None });
    test_value_le(
        &v,
        &[
            16, 0, 0, 0, // U8Map
            1, 0, 0, 0, 0, 0, 0, 0,    // length
            0x12, // key 0
            0, 0, 0, 0, // value 0
        ],
    );
    test_value_be(
        &v,
        &[
            0, 0, 0, 16, // U8Map
            0, 0, 0, 0, 0, 0, 0, 1,    // length
            0x12, // key 0
            0, 0, 0, 0, // value 0
        ],
    );
}

#[test]
fn value_i8_map() {
    let v = Value::I8Map(hashmap! {});
    test_value_le(
        &v,
        &[
            17, 0, 0, 0, // I8Map
            0, 0, 0, 0, 0, 0, 0, 0, // length
        ],
    );
    test_value_be(
        &v,
        &[
            0, 0, 0, 17, // I8Map
            0, 0, 0, 0, 0, 0, 0, 0, // length
        ],
    );
    let v = Value::I8Map(hashmap! { 0x12 => Value::None });
    test_value_le(
        &v,
        &[
            17, 0, 0, 0, // I8Map
            1, 0, 0, 0, 0, 0, 0, 0,    // length
            0x12, // key 0
            0, 0, 0, 0, // value 0
        ],
    );
    test_value_be(
        &v,
        &[
            0, 0, 0, 17, // I8Map
            0, 0, 0, 0, 0, 0, 0, 1,    // length
            0x12, // key 0
            0, 0, 0, 0, // value 0
        ],
    );
}

#[test]
fn value_u16_map() {
    let v = Value::U16Map(hashmap! {});
    test_value_le(
        &v,
        &[
            18, 0, 0, 0, // U16Map
            0, 0, 0, 0, 0, 0, 0, 0, // length
        ],
    );
    test_value_be(
        &v,
        &[
            0, 0, 0, 18, // U16Map
            0, 0, 0, 0, 0, 0, 0, 0, // length
        ],
    );
    let v = Value::U16Map(hashmap! { 0x1234 => Value::None });
    test_value_le(
        &v,
        &[
            18, 0, 0, 0, // U16Map
            1, 0, 0, 0, 0, 0, 0, 0, // length
            0x34, 0x12, // key 0
            0, 0, 0, 0, // value 0
        ],
    );
    test_value_be(
        &v,
        &[
            0, 0, 0, 18, // U16Map
            0, 0, 0, 0, 0, 0, 0, 1, // length
            0x12, 0x34, // key 0
            0, 0, 0, 0, // value 0
        ],
    );
}

#[test]
fn value_i16_map() {
    let v = Value::I16Map(hashmap! {});
    test_value_le(
        &v,
        &[
            19, 0, 0, 0, // I16Map
            0, 0, 0, 0, 0, 0, 0, 0, // length
        ],
    );
    test_value_be(
        &v,
        &[
            0, 0, 0, 19, // I16Map
            0, 0, 0, 0, 0, 0, 0, 0, // length
        ],
    );
    let v = Value::I16Map(hashmap! { 0x1234 => Value::None });
    test_value_le(
        &v,
        &[
            19, 0, 0, 0, // I16Map
            1, 0, 0, 0, 0, 0, 0, 0, // length
            0x34, 0x12, // key 0
            0, 0, 0, 0, // value 0
        ],
    );
    test_value_be(
        &v,
        &[
            0, 0, 0, 19, // I16Map
            0, 0, 0, 0, 0, 0, 0, 1, // length
            0x12, 0x34, // key 0
            0, 0, 0, 0, // value 0
        ],
    );
}

#[test]
fn value_u32_map() {
    let v = Value::U32Map(hashmap! {});
    test_value_le(
        &v,
        &[
            20, 0, 0, 0, // U32Map
            0, 0, 0, 0, 0, 0, 0, 0, // length
        ],
    );
    test_value_be(
        &v,
        &[
            0, 0, 0, 20, // U32Map
            0, 0, 0, 0, 0, 0, 0, 0, // length
        ],
    );
    let v = Value::U32Map(hashmap! { 0x12345678 => Value::None });
    test_value_le(
        &v,
        &[
            20, 0, 0, 0, // U32Map
            1, 0, 0, 0, 0, 0, 0, 0, // length
            0x78, 0x56, 0x34, 0x12, // key 0
            0, 0, 0, 0, // value 0
        ],
    );
    test_value_be(
        &v,
        &[
            0, 0, 0, 20, // U32Map
            0, 0, 0, 0, 0, 0, 0, 1, // length
            0x12, 0x34, 0x56, 0x78, // key 0
            0, 0, 0, 0, // value 0
        ],
    );
}

#[test]
fn value_i32_map() {
    let v = Value::I32Map(hashmap! {});
    test_value_le(
        &v,
        &[
            21, 0, 0, 0, // I32Map
            0, 0, 0, 0, 0, 0, 0, 0, // length
        ],
    );
    test_value_be(
        &v,
        &[
            0, 0, 0, 21, // I32Map
            0, 0, 0, 0, 0, 0, 0, 0, // length
        ],
    );
    let v = Value::I32Map(hashmap! { 0x12345678 => Value::None });
    test_value_le(
        &v,
        &[
            21, 0, 0, 0, // I32Map
            1, 0, 0, 0, 0, 0, 0, 0, // length
            0x78, 0x56, 0x34, 0x12, // key 0
            0, 0, 0, 0, // value 0
        ],
    );
    test_value_be(
        &v,
        &[
            0, 0, 0, 21, // I32Map
            0, 0, 0, 0, 0, 0, 0, 1, // length
            0x12, 0x34, 0x56, 0x78, // key 0
            0, 0, 0, 0, // value 0
        ],
    );
}

#[test]
fn value_u64_map() {
    let v = Value::U64Map(hashmap! {});
    test_value_le(
        &v,
        &[
            22, 0, 0, 0, // U64Map
            0, 0, 0, 0, 0, 0, 0, 0, // length
        ],
    );
    test_value_be(
        &v,
        &[
            0, 0, 0, 22, // U64Map
            0, 0, 0, 0, 0, 0, 0, 0, // length
        ],
    );
    let v = Value::U64Map(hashmap! { 0x123456789abcdef0 => Value::None });
    test_value_le(
        &v,
        &[
            22, 0, 0, 0, // U64Map
            1, 0, 0, 0, 0, 0, 0, 0, // length
            0xf0, 0xde, 0xbc, 0x9a, 0x78, 0x56, 0x34, 0x12, // key 0
            0, 0, 0, 0, // value 0
        ],
    );
    test_value_be(
        &v,
        &[
            0, 0, 0, 22, // U64Map
            0, 0, 0, 0, 0, 0, 0, 1, // length
            0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, // key 0
            0, 0, 0, 0, // value 0
        ],
    );
}

#[test]
fn value_i64_map() {
    let v = Value::I64Map(hashmap! {});
    test_value_le(
        &v,
        &[
            23, 0, 0, 0, // I64Map
            0, 0, 0, 0, 0, 0, 0, 0, // length
        ],
    );
    test_value_be(
        &v,
        &[
            0, 0, 0, 23, // I64Map
            0, 0, 0, 0, 0, 0, 0, 0, // length
        ],
    );
    let v = Value::I64Map(hashmap! { 0x123456789abcdef0 => Value::None });
    test_value_le(
        &v,
        &[
            23, 0, 0, 0, // I64Map
            1, 0, 0, 0, 0, 0, 0, 0, // length
            0xf0, 0xde, 0xbc, 0x9a, 0x78, 0x56, 0x34, 0x12, // key 0
            0, 0, 0, 0, // value 0
        ],
    );
    test_value_be(
        &v,
        &[
            0, 0, 0, 23, // I64Map
            0, 0, 0, 0, 0, 0, 0, 1, // length
            0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, // key 0
            0, 0, 0, 0, // value 0
        ],
    );
}

#[test]
fn value_string_map() {
    let v = Value::StringMap(hashmap! {});
    test_value_le(
        &v,
        &[
            24, 0, 0, 0, // StringMap
            0, 0, 0, 0, 0, 0, 0, 0, // length
        ],
    );
    test_value_be(
        &v,
        &[
            0, 0, 0, 24, // StringMap
            0, 0, 0, 0, 0, 0, 0, 0, // length
        ],
    );
    let v = Value::StringMap(hashmap! { "aldrin".to_owned() => Value::None });
    test_value_le(
        &v,
        &[
            24, 0, 0, 0, // StringMap
            1, 0, 0, 0, 0, 0, 0, 0, // length
            6, 0, 0, 0, 0, 0, 0, 0, // length key 0
            b'a', b'l', b'd', b'r', b'i', b'n', // value key 0
            0, 0, 0, 0, // value 0
        ],
    );
    test_value_be(
        &v,
        &[
            0, 0, 0, 24, // StringMap
            0, 0, 0, 0, 0, 0, 0, 1, // length
            0, 0, 0, 0, 0, 0, 0, 6, // length key 0
            b'a', b'l', b'd', b'r', b'i', b'n', // value key 0
            0, 0, 0, 0, // value 0
        ],
    );
}

#[test]
fn value_uuid_map() {
    let v = Value::UuidMap(hashmap! {});
    test_value_le(
        &v,
        &[
            25, 0, 0, 0, // UuidMap
            0, 0, 0, 0, 0, 0, 0, 0, // length
        ],
    );
    test_value_be(
        &v,
        &[
            0, 0, 0, 25, // UuidMap
            0, 0, 0, 0, 0, 0, 0, 0, // length
        ],
    );
    let v = Value::UuidMap(
        hashmap! { Uuid::from_u128(0x00112233445566778899aabbccddeeff) => Value::None },
    );
    test_value_le(
        &v,
        &[
            25, 0, 0, 0, // UuidMap
            1, 0, 0, 0, 0, 0, 0, 0, // length
            16, 0, 0, 0, 0, 0, 0, 0, // uuid length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // key 0
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // key 0
            0, 0, 0, 0, // value 0
        ],
    );
    test_value_be(
        &v,
        &[
            0, 0, 0, 25, // UuidMap
            0, 0, 0, 0, 0, 0, 0, 1, // length
            0, 0, 0, 0, 0, 0, 0, 16, // uuid length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // key 0
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // key 0
            0, 0, 0, 0, // value 0
        ],
    );
}

#[test]
fn value_u8_set() {
    let v = Value::U8Set(hashset! {});
    test_value_le(
        &v,
        &[
            26, 0, 0, 0, // U8Set
            0, 0, 0, 0, 0, 0, 0, 0, // length
        ],
    );
    test_value_be(
        &v,
        &[
            0, 0, 0, 26, // U8Set
            0, 0, 0, 0, 0, 0, 0, 0, // length
        ],
    );
    let v = Value::U8Set(hashset! { 0x12 });
    test_value_le(
        &v,
        &[
            26, 0, 0, 0, // U8Set
            1, 0, 0, 0, 0, 0, 0, 0,    // length
            0x12, // value 0
        ],
    );
    test_value_be(
        &v,
        &[
            0, 0, 0, 26, // U8Set
            0, 0, 0, 0, 0, 0, 0, 1,    // length
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
            27, 0, 0, 0, // I8Set
            0, 0, 0, 0, 0, 0, 0, 0, // length
        ],
    );
    test_value_be(
        &v,
        &[
            0, 0, 0, 27, // I8Set
            0, 0, 0, 0, 0, 0, 0, 0, // length
        ],
    );
    let v = Value::I8Set(hashset! { 0x12 });
    test_value_le(
        &v,
        &[
            27, 0, 0, 0, // I8Set
            1, 0, 0, 0, 0, 0, 0, 0,    // length
            0x12, // value 0
        ],
    );
    test_value_be(
        &v,
        &[
            0, 0, 0, 27, // I8Set
            0, 0, 0, 0, 0, 0, 0, 1,    // length
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
            28, 0, 0, 0, // U16Set
            0, 0, 0, 0, 0, 0, 0, 0, // length
        ],
    );
    test_value_be(
        &v,
        &[
            0, 0, 0, 28, // U16Set
            0, 0, 0, 0, 0, 0, 0, 0, // length
        ],
    );
    let v = Value::U16Set(hashset! { 0x1234 });
    test_value_le(
        &v,
        &[
            28, 0, 0, 0, // U16Set
            1, 0, 0, 0, 0, 0, 0, 0, // length
            0x34, 0x12, // value 0
        ],
    );
    test_value_be(
        &v,
        &[
            0, 0, 0, 28, // U16Set
            0, 0, 0, 0, 0, 0, 0, 1, // length
            0x12, 0x34, // value 0
        ],
    );
}

#[test]
fn value_i16_set() {
    let v = Value::I16Set(hashset! {});
    test_value_le(
        &v,
        &[
            29, 0, 0, 0, // I16Set
            0, 0, 0, 0, 0, 0, 0, 0, // length
        ],
    );
    test_value_be(
        &v,
        &[
            0, 0, 0, 29, // I16Set
            0, 0, 0, 0, 0, 0, 0, 0, // length
        ],
    );
    let v = Value::I16Set(hashset! { 0x1234 });
    test_value_le(
        &v,
        &[
            29, 0, 0, 0, // I16Set
            1, 0, 0, 0, 0, 0, 0, 0, // length
            0x34, 0x12, // value 0
        ],
    );
    test_value_be(
        &v,
        &[
            0, 0, 0, 29, // I16Set
            0, 0, 0, 0, 0, 0, 0, 1, // length
            0x12, 0x34, // value 0
        ],
    );
}

#[test]
fn value_u32_set() {
    let v = Value::U32Set(hashset! {});
    test_value_le(
        &v,
        &[
            30, 0, 0, 0, // U32Set
            0, 0, 0, 0, 0, 0, 0, 0, // length
        ],
    );
    test_value_be(
        &v,
        &[
            0, 0, 0, 30, // U32Set
            0, 0, 0, 0, 0, 0, 0, 0, // length
        ],
    );
    let v = Value::U32Set(hashset! { 0x12345678 });
    test_value_le(
        &v,
        &[
            30, 0, 0, 0, // U32Set
            1, 0, 0, 0, 0, 0, 0, 0, // length
            0x78, 0x56, 0x34, 0x12, // value 0
        ],
    );
    test_value_be(
        &v,
        &[
            0, 0, 0, 30, // U32Set
            0, 0, 0, 0, 0, 0, 0, 1, // length
            0x12, 0x34, 0x56, 0x78, // value 0
        ],
    );
}

#[test]
fn value_i32_set() {
    let v = Value::I32Set(hashset! {});
    test_value_le(
        &v,
        &[
            31, 0, 0, 0, // I32Set
            0, 0, 0, 0, 0, 0, 0, 0, // length
        ],
    );
    test_value_be(
        &v,
        &[
            0, 0, 0, 31, // I32Set
            0, 0, 0, 0, 0, 0, 0, 0, // length
        ],
    );
    let v = Value::I32Set(hashset! { 0x12345678 });
    test_value_le(
        &v,
        &[
            31, 0, 0, 0, // I32Set
            1, 0, 0, 0, 0, 0, 0, 0, // length
            0x78, 0x56, 0x34, 0x12, // value 0
        ],
    );
    test_value_be(
        &v,
        &[
            0, 0, 0, 31, // I32Set
            0, 0, 0, 0, 0, 0, 0, 1, // length
            0x12, 0x34, 0x56, 0x78, // value 0
        ],
    );
}

#[test]
fn value_u64_set() {
    let v = Value::U64Set(hashset! {});
    test_value_le(
        &v,
        &[
            32, 0, 0, 0, // U64Set
            0, 0, 0, 0, 0, 0, 0, 0, // length
        ],
    );
    test_value_be(
        &v,
        &[
            0, 0, 0, 32, // U64Set
            0, 0, 0, 0, 0, 0, 0, 0, // length
        ],
    );
    let v = Value::U64Set(hashset! { 0x123456789abcdef0 });
    test_value_le(
        &v,
        &[
            32, 0, 0, 0, // U64Set
            1, 0, 0, 0, 0, 0, 0, 0, // length
            0xf0, 0xde, 0xbc, 0x9a, 0x78, 0x56, 0x34, 0x12, // value 0
        ],
    );
    test_value_be(
        &v,
        &[
            0, 0, 0, 32, // U64Set
            0, 0, 0, 0, 0, 0, 0, 1, // length
            0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, // value 0
        ],
    );
}

#[test]
fn value_i64_set() {
    let v = Value::I64Set(hashset! {});
    test_value_le(
        &v,
        &[
            33, 0, 0, 0, // I64Set
            0, 0, 0, 0, 0, 0, 0, 0, // length
        ],
    );
    test_value_be(
        &v,
        &[
            0, 0, 0, 33, // I64Set
            0, 0, 0, 0, 0, 0, 0, 0, // length
        ],
    );
    let v = Value::I64Set(hashset! { 0x123456789abcdef0 });
    test_value_le(
        &v,
        &[
            33, 0, 0, 0, // I64Set
            1, 0, 0, 0, 0, 0, 0, 0, // length
            0xf0, 0xde, 0xbc, 0x9a, 0x78, 0x56, 0x34, 0x12, // value 0
        ],
    );
    test_value_be(
        &v,
        &[
            0, 0, 0, 33, // I64Set
            0, 0, 0, 0, 0, 0, 0, 1, // length
            0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, // value 0
        ],
    );
}

#[test]
fn value_string_set() {
    let v = Value::StringSet(hashset! {});
    test_value_le(
        &v,
        &[
            34, 0, 0, 0, // StringSet
            0, 0, 0, 0, 0, 0, 0, 0, // length
        ],
    );
    test_value_be(
        &v,
        &[
            0, 0, 0, 34, // StringSet
            0, 0, 0, 0, 0, 0, 0, 0, // length
        ],
    );
    let v = Value::StringSet(hashset! { "aldrin".to_owned() });
    test_value_le(
        &v,
        &[
            34, 0, 0, 0, // StringSet
            1, 0, 0, 0, 0, 0, 0, 0, // length
            6, 0, 0, 0, 0, 0, 0, 0, // length value 0
            b'a', b'l', b'd', b'r', b'i', b'n', // value key 0
        ],
    );
    test_value_be(
        &v,
        &[
            0, 0, 0, 34, // StringSet
            0, 0, 0, 0, 0, 0, 0, 1, // length
            0, 0, 0, 0, 0, 0, 0, 6, // length value 0
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
            35, 0, 0, 0, // UuidSet
            0, 0, 0, 0, 0, 0, 0, 0, // length
        ],
    );
    test_value_be(
        &v,
        &[
            0, 0, 0, 35, // UuidSet
            0, 0, 0, 0, 0, 0, 0, 0, // length
        ],
    );
    let v = Value::UuidSet(hashset! { Uuid::from_u128(0x00112233445566778899aabbccddeeff) });
    test_value_le(
        &v,
        &[
            35, 0, 0, 0, // UuidSet
            1, 0, 0, 0, 0, 0, 0, 0, // length
            16, 0, 0, 0, 0, 0, 0, 0, // uuid length
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // key 0
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, // key 0
        ],
    );
    test_value_be(
        &v,
        &[
            0, 0, 0, 35, // UuidSet
            0, 0, 0, 0, 0, 0, 0, 1, // length
            0, 0, 0, 0, 0, 0, 0, 16, // uuid length
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
            36, 0, 0, 0, // Struct
            0, 0, 0, 0, 0, 0, 0, 0, // length
        ],
    );
    test_value_be(
        &v,
        &[
            0, 0, 0, 36, // Struct
            0, 0, 0, 0, 0, 0, 0, 0, // length
        ],
    );
    let v = Value::Struct(hashmap! { 0x12345678 => Value::None });
    test_value_le(
        &v,
        &[
            36, 0, 0, 0, // Struct
            1, 0, 0, 0, 0, 0, 0, 0, // length
            0x78, 0x56, 0x34, 0x12, // key 0
            0, 0, 0, 0, // value 0
        ],
    );
    test_value_be(
        &v,
        &[
            0, 0, 0, 36, // Struct
            0, 0, 0, 0, 0, 0, 0, 1, // length
            0x12, 0x34, 0x56, 0x78, // key 0
            0, 0, 0, 0, // value 0
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
            37, 0, 0, 0, // Enum
            0x78, 0x56, 0x34, 0x12, // variant
            0, 0, 0, 0, // value
        ],
    );
    test_value_be(
        &v,
        &[
            0, 0, 0, 37, // Enum
            0x12, 0x34, 0x56, 0x78, // variant
            0, 0, 0, 0, // value
        ],
    );
}
