aldrin_client::generate!("test/channels.aldrin");
aldrin_client::generate!("test/constants.aldrin");
aldrin_client::generate!("test/restore_value_on_error.aldrin");
aldrin_client::generate!("test/test1.aldrin");

use aldrin_client::Error;
use aldrin_proto::ObjectUuid;
use aldrin_test::tokio_based::TestBroker;
use futures::StreamExt;
use uuid::uuid;

#[tokio::test]
async fn auto_reply_with_invalid_args() {
    let broker = TestBroker::new();
    let client = broker.add_client().await;

    let obj = client.create_object(ObjectUuid::new_v4()).await.unwrap();
    let mut svc = test1::Test1::create(&obj).await.unwrap();
    let id = svc.id();
    tokio::spawn(async move { while svc.next().await.is_some() {} });

    let res = client
        .call_infallible_function::<u32, ()>(id, 1, &0)
        .unwrap()
        .await;
    assert_eq!(res, Err(Error::InvalidArgs(id, 1)));

    let res = client
        .call_infallible_function::<(), ()>(id, 2, &())
        .unwrap()
        .await;
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

    let res = client
        .call_infallible_function::<(), ()>(id, 3, &())
        .unwrap()
        .await;
    assert_eq!(res, Err(Error::InvalidFunction(id, 3)));
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