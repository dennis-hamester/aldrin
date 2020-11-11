aldrin_client::generate!("test/test1.aldrin");
aldrin_client::generate!("test/restore_value_on_error.aldrin");

use aldrin_client::{Error, ObjectUuid};
use aldrin_proto::{FromValue, Value};
use aldrin_test::tokio_based::TestBroker;
use futures::StreamExt;

#[tokio::test]
async fn auto_reply_with_invalid_args() {
    let broker = TestBroker::new();
    let client = broker.add_client().await;

    let obj = client.create_object(ObjectUuid::new_v4()).await.unwrap();
    let mut svc = test1::Test1::create(&obj).await.unwrap();
    let id = svc.id();
    tokio::spawn(async move { while svc.next().await.is_some() {} });

    let res = client.call_function(id, 1, Value::U32(0)).unwrap().await;
    assert_eq!(res, Err(Error::InvalidArgs(id, 1)));

    let res = client.call_function(id, 2, Value::None).unwrap().await;
    assert_eq!(res, Err(Error::InvalidArgs(id, 2)));
}

#[tokio::test]
async fn auto_reply_with_invalid_function() {
    let broker = TestBroker::new();
    let client = broker.add_client().await;

    let obj = client.create_object(ObjectUuid::new_v4()).await.unwrap();
    let mut svc = test1::Test1::create(&obj).await.unwrap();
    let id = svc.id();
    tokio::spawn(async move { while svc.next().await.is_some() {} });

    let res = client.call_function(id, 3, Value::None).unwrap().await;
    assert_eq!(res, Err(Error::InvalidFunction(id, 3)));
}

#[test]
fn restore_enum_on_error() {
    // wrong type
    let before = Value::U8(0);
    let after = restore_value_on_error::Enum::from_value(before.clone()).unwrap_err();
    assert_eq!(before, after.0.unwrap());

    // invalid variant
    let before = Value::Enum {
        variant: 0,
        value: Box::new(Value::None),
    };
    let after = restore_value_on_error::Enum::from_value(before.clone()).unwrap_err();
    assert_eq!(before, after.0.unwrap());

    // wrong type for Variant1
    let before = Value::Enum {
        variant: 1,
        value: Box::new(Value::U8(0)),
    };
    let after = restore_value_on_error::Enum::from_value(before.clone()).unwrap_err();
    assert_eq!(before, after.0.unwrap());

    // wrong type for Variant2
    let before = Value::Enum {
        variant: 2,
        value: Box::new(Value::U8(0)),
    };
    let after = restore_value_on_error::Enum::from_value(before.clone()).unwrap_err();
    assert_eq!(before, after.0.unwrap());

    // wrong type for Variant3
    let before = Value::Enum {
        variant: 3,
        value: Box::new(Value::U8(0)),
    };
    let after = restore_value_on_error::Enum::from_value(before.clone()).unwrap_err();
    assert_eq!(before, after.0.unwrap());
}
