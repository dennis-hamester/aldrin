aldrin_client::generate!("test/test1.aldrin");

use aldrin_client::{Error, ObjectUuid};
use aldrin_proto::Value;
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
