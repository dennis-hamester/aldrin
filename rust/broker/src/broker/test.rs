use crate::Broker;
use aldrin_client::{Client, ObjectUuid, ServiceUuid};
use aldrin_proto::{AsyncTransportExt, Connect, ConnectReply, Message, Value};
use aldrin_test::tokio_based::TestBroker;
use futures_util::future::{self, Either};
use futures_util::stream::StreamExt;
use std::future::Future;
use std::mem;
use std::time::Duration;
use tokio::time;

#[tokio::test]
async fn disconnect_during_function_call() {
    let mut broker = TestBroker::new();

    let mut client1 = broker.add_client().await;
    let obj = client1.create_object(ObjectUuid::new_v4()).await.unwrap();
    let mut svc = obj.create_service(ServiceUuid::new_v4(), 0).await.unwrap();

    // client2 calls a function on client1 and disconnects before client1 replies.
    let mut client2 = broker.add_client().await;
    let _ = client2
        .call_infallible_function::<(), ()>(svc.id(), 0, ())
        .unwrap();
    client2.join().await;

    let call = svc.next().await.unwrap();
    call.reply.ok(()).unwrap();
    client1.join().await;

    broker.join_idle().await
}

#[tokio::test]
async fn drop_conn_before_function_call() {
    let mut broker = TestBroker::new();

    // Setup client1 manually, such that we can drop its connection future (conn1_fut) at the right
    // moment.
    let (t1, t2) = aldrin_channel::unbounded();
    let client1_fut = Client::connect(t1);
    let conn1_fut = broker.connect(t2);
    let (client1_fut, conn1_fut) = future::join(client1_fut, conn1_fut).await;
    let client1_fut = client1_fut.unwrap();
    let client1 = client1_fut.handle().clone();
    tokio::spawn(client1_fut.run());
    let mut conn1_fut = Box::pin(conn1_fut.unwrap().run());

    let client2 = broker.add_client().await;

    async fn select_first<F1: Future, F2: Future>(f1: F1, f2: F2) -> F1::Output {
        match future::select(Box::pin(f1), Box::pin(f2)).await {
            Either::Left((res, _)) => res,
            Either::Right(_) => unreachable!(),
        }
    }

    let obj = client1.create_object(ObjectUuid::new_v4());
    let obj = select_first(obj, &mut conn1_fut).await.unwrap();

    let svc = obj.create_service(ServiceUuid::new_v4(), 0);
    let svc = select_first(svc, &mut conn1_fut).await.unwrap();

    // This will cause all subsequent sends in the broker to fail.
    mem::drop(conn1_fut);

    // Calling a function on conn1 must not deadlock, but be immediately replied to with an error.
    let res = time::timeout(
        Duration::from_millis(500),
        client2
            .call_infallible_function::<(), ()>(svc.id(), 0, ())
            .unwrap(),
    )
    .await
    .unwrap();

    assert!(res.is_err());
}

#[tokio::test]
async fn begin_connect_accept() {
    let broker = Broker::new();
    let mut handle = broker.handle().clone();
    let join = tokio::spawn(broker.run());

    let (mut t1, t2) = aldrin_channel::unbounded();

    t1.send_and_flush(Message::Connect(Connect {
        version: aldrin_proto::VERSION,
        data: Value::U32(0),
    }))
    .await
    .unwrap();

    let mut conn = handle.begin_connect(t2).await.unwrap();
    assert_eq!(conn.take_client_data(), Value::U32(0));

    let _ = conn.accept(Value::U32(1)).await.unwrap();
    let msg = t1.receive().await.unwrap();
    assert_eq!(msg, Message::ConnectReply(ConnectReply::Ok(Value::U32(1))));

    handle.shutdown().await;
    join.await.unwrap();
}

#[tokio::test]
async fn begin_connect_reject() {
    let broker = Broker::new();
    let mut handle = broker.handle().clone();
    let join = tokio::spawn(broker.run());

    let (mut t1, t2) = aldrin_channel::unbounded();

    t1.send_and_flush(Message::Connect(Connect {
        version: aldrin_proto::VERSION,
        data: Value::U32(0),
    }))
    .await
    .unwrap();

    let mut conn = handle.begin_connect(t2).await.unwrap();
    assert_eq!(conn.take_client_data(), Value::U32(0));

    conn.reject(Value::U32(1)).await.unwrap();
    let msg = t1.receive().await.unwrap();
    assert_eq!(
        msg,
        Message::ConnectReply(ConnectReply::Rejected(Value::U32(1)))
    );

    handle.shutdown().await;
    join.await.unwrap();
}
