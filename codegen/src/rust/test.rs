aldrin::generate!("test/constants.aldrin");
aldrin::generate!("test/generic_struct.aldrin");
aldrin::generate!("test/introspection.aldrin", introspection = true);
aldrin::generate!("test/old_new.aldrin");
aldrin::generate!("test/options.aldrin");
aldrin::generate!("test/result.aldrin");
aldrin::generate!("test/subscribe_all.aldrin");
aldrin::generate!("test/test1.aldrin");
aldrin::generate!("test/unit.aldrin");

mod conditional_introspection {
    mod available {
        aldrin::generate!("test/introspection.aldrin", introspection_if = "rust");
    }

    mod unavailable {
        aldrin::generate!("test/introspection.aldrin", introspection_if = "disabled");
    }
}

use aldrin::core::{ObjectUuid, SerializedValue};
use aldrin::low_level::Proxy;
use aldrin::Error;
use aldrin_test::tokio::TestBroker;
use futures_util::stream::StreamExt;
use subscribe_all::SubscribeAllEvent;
use uuid::uuid;

#[tokio::test]
async fn auto_reply_with_invalid_args() {
    let mut broker = TestBroker::new();
    let client = broker.add_client().await;

    let obj = client.create_object(ObjectUuid::new_v4()).await.unwrap();
    let mut svc = test1::Test1::new(&obj).await.unwrap();
    let proxy = Proxy::new(&client, svc.id()).await.unwrap();
    tokio::spawn(async move { while svc.next().await.is_some() {} });

    let res = proxy.call(1, &0).await;
    assert_eq!(res, Err(Error::invalid_arguments(1, None)));

    let res = proxy.call(2, &()).await;
    assert_eq!(res, Err(Error::invalid_arguments(2, None)));
}

#[tokio::test]
async fn auto_reply_with_invalid_function() {
    let mut broker = TestBroker::new();
    let client = broker.add_client().await;

    let obj = client.create_object(ObjectUuid::new_v4()).await.unwrap();
    let mut svc = test1::Test1::new(&obj).await.unwrap();
    let proxy = Proxy::new(&client, svc.id()).await.unwrap();
    tokio::spawn(async move { while svc.next().await.is_some() {} });

    let res = proxy.call(3, &()).await;
    assert_eq!(res, Err(Error::invalid_function(3)));
}

#[test]
fn constants() {
    assert_eq!(constants::CONST_U8, 1);
    assert_eq!(constants::CONST_I8, 2);
    assert_eq!(constants::CONST_U16, 3);
    assert_eq!(constants::CONST_I16, 4);
    assert_eq!(constants::CONST_U32, 5);
    assert_eq!(constants::CONST_I32, 6);
    assert_eq!(constants::CONST_U64, 7);
    assert_eq!(constants::CONST_I64, 8);
    assert_eq!(constants::CONST_STRING, "string");
    assert_eq!(
        constants::CONST_UUID,
        uuid!("5c368dc9-e6d3-4545-86d1-435fe3e771cc")
    );
}

#[test]
fn generic_struct() {
    let s1 = generic_struct::Struct {
        field1: 1,
        field2: None,
    };

    let s1_serialized = SerializedValue::serialize(&s1).unwrap();
    let g: aldrin::core::Struct = s1_serialized.deserialize().unwrap();
    let g_serialized = SerializedValue::serialize(&g).unwrap();
    let s2: generic_struct::Struct = g_serialized.deserialize().unwrap();

    assert_eq!(s1, s2);
}

#[test]
fn old_as_new() {
    let old = old_new::Old { f1: 1 };
    let old_serialized = SerializedValue::serialize(&old).unwrap();
    let new: old_new::New = old_serialized.deserialize().unwrap();
    assert_eq!(new.f1, 1);
    assert_eq!(new.f2, None);
}

#[test]
fn new_as_old() {
    let new = old_new::New { f1: 1, f2: None };
    let new_serialized = SerializedValue::serialize(&new).unwrap();
    let old: old_new::Old = new_serialized.deserialize().unwrap();
    assert_eq!(old.f1, 1);

    let new = old_new::New { f1: 1, f2: Some(2) };
    let new_serialized = SerializedValue::serialize(&new).unwrap();
    let old: old_new::Old = new_serialized.deserialize().unwrap();
    assert_eq!(old.f1, 1);
}

#[tokio::test]
async fn unsubscribe_all() {
    let mut broker = TestBroker::new();
    let client = broker.add_client().await;

    let obj = client.create_object(ObjectUuid::new_v4()).await.unwrap();
    let svc = subscribe_all::SubscribeAll::new(&obj).await.unwrap();
    let mut proxy = subscribe_all::SubscribeAllProxy::new(&client, svc.id())
        .await
        .unwrap();

    proxy.subscribe_ev1().await.unwrap();
    proxy.inner().subscribe_all().await.unwrap();

    svc.ev1().unwrap();
    svc.ev2().unwrap();

    assert!(matches!(
        proxy.next_event().await,
        Some(Ok(SubscribeAllEvent::Ev1))
    ));

    assert!(matches!(
        proxy.next_event().await,
        Some(Ok(SubscribeAllEvent::Ev2))
    ));

    proxy.unsubscribe_all().await.unwrap();

    svc.ev1().unwrap();
    svc.ev2().unwrap();
    svc.destroy().await.unwrap();

    assert!(proxy.next_event().await.is_none());
}
