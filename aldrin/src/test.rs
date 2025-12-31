use crate::Client;
use aldrin_broker::Broker;
use aldrin_core::{channel, ObjectUuid, ServiceUuid};
use aldrin_test::aldrin::low_level::{Proxy, ServiceInfo};
use aldrin_test::tokio::TestBroker;
use std::mem;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::time;

struct PollOnce<Fut>(Fut);

impl<Fut: Future + Unpin> Future for PollOnce<Fut> {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let _ = Pin::new(&mut self.0).poll(cx);
        Poll::Ready(())
    }
}

#[tokio::test]
async fn client_stops_when_last_handle_is_dropped() {
    let mut broker = aldrin_test::TestBroker::new();
    tokio::spawn(broker.take_broker().run());

    let mut client = broker.add_client().await;
    tokio::spawn(client.take_connection().run());
    let join = tokio::spawn(client.take_client().run());

    let _ = client.clone();
    let _ = client.clone();
    let _ = client.clone();
    let _ = client.clone();
    mem::drop(client);

    time::timeout(Duration::from_millis(100), join)
        .await
        .unwrap()
        .unwrap()
        .unwrap();
}

#[tokio::test]
async fn abort_create_object() {
    let mut broker = TestBroker::new();
    let client = broker.add_client().await;

    let uuid = ObjectUuid::new_v4();
    let fut = client.create_object(uuid);

    // This assumes that polling the future once is enough to create the object.
    PollOnce(Box::pin(fut)).await;

    // The object may have been created temporarily. Give client and broker some time to destroy it
    // again.
    time::sleep(Duration::from_millis(100)).await;

    assert!(client.create_object(uuid).await.is_ok());
}

#[tokio::test]
async fn abort_create_service() {
    let mut broker = TestBroker::new();
    let client = broker.add_client().await;

    let obj = client.create_object(ObjectUuid::new_v4()).await.unwrap();
    let uuid = ServiceUuid::new_v4();
    let info = ServiceInfo::new(0);
    let fut = obj.create_service(uuid, info);

    // This assumes that polling the future once is enough to create the service.
    PollOnce(Box::pin(fut)).await;

    // The service may have been created temporarily. Give client and broker some time to destroy it
    // again.
    time::sleep(Duration::from_millis(100)).await;

    assert!(obj.create_service(uuid, info).await.is_ok());
}

#[tokio::test]
async fn transport_error_before_client_shutdown() {
    let mut broker = aldrin_test::TestBroker::new();
    tokio::spawn(broker.take_broker().run());

    let mut client = broker.add_client().await;
    let join = tokio::spawn(client.take_client().run());

    // Drop the connnection.
    let _ = client.take_connection();

    // Issue a client shutdown.
    client.shutdown();

    // Client future must complete with an error.
    let res = join.await.unwrap();
    assert!(res.is_err());
}

#[tokio::test]
async fn transport_error_after_client_shutdown() {
    let mut broker = aldrin_test::TestBroker::new();
    tokio::spawn(broker.take_broker().run());

    let mut client = broker.add_client().await;
    let join = tokio::spawn(client.take_client().run());

    // Issue a client shutdown.
    client.shutdown();
    tokio::task::yield_now().await;

    // Drop the connnection.
    let _ = client.take_connection();

    // Client future must complete with an error.
    let res = join.await.unwrap();
    assert!(res.is_err());
}

#[tokio::test]
async fn abort_function_call() {
    let mut broker = TestBroker::new();
    let mut client = broker.add_client().await;

    let obj = client.create_object(ObjectUuid::new_v4()).await.unwrap();
    let info = ServiceInfo::new(0);
    let mut svc = obj
        .create_service(ServiceUuid::new_v4(), info)
        .await
        .unwrap();
    let proxy = Proxy::new(&client, svc.id()).await.unwrap();

    let reply = proxy.call(0, (), None);
    let mut promise = svc.next_call().await.unwrap().into_promise();

    assert!(!promise.is_aborted());
    reply.abort();
    promise.aborted().await;
    assert!(promise.is_aborted());
    promise.aborted().await;

    client.join().await;
    broker.join().await;
}

#[tokio::test]
async fn reply_aborted_call() {
    let mut broker = TestBroker::new();
    let mut client = broker.add_client().await;

    let obj = client.create_object(ObjectUuid::new_v4()).await.unwrap();
    let info = ServiceInfo::new(0);
    let mut svc = obj
        .create_service(ServiceUuid::new_v4(), info)
        .await
        .unwrap();
    let proxy = Proxy::new(&client, svc.id()).await.unwrap();

    let reply = proxy.call(0, (), None);
    let mut promise = svc.next_call().await.unwrap().into_promise();

    assert!(!promise.is_aborted());
    reply.abort();
    promise.aborted().await;
    assert!(promise.is_aborted());
    promise.done().unwrap();

    client.join().await;
    broker.join().await;
}

#[tokio::test]
async fn clean_shutdown_from_broker() {
    let mut broker = TestBroker::new();
    let mut client = broker.add_client().await;

    broker.join().await;
    client.join().await;
}

#[tokio::test]
async fn bounded_channel_deadlock() {
    let (mut broker, broker_join) = {
        let broker = Broker::new();
        let handle = broker.handle().clone();
        (handle, tokio::spawn(broker.run()))
    };

    let (client, client_join) = {
        let (ch1, ch2) = channel::bounded(1);
        let (client, conn) = tokio::join!(Client::connect(ch1), broker.connect(ch2));

        let client = client.unwrap();
        let handle = client.handle().clone();
        let client_join = tokio::spawn(client.run());

        let conn = conn.unwrap();
        tokio::spawn(conn.run());

        (handle, client_join)
    };

    let _obj = client.create_object(ObjectUuid::new_v4()).await.unwrap();

    let deadlock = async {
        tokio::try_join!(
            client.find_object_n(None, &[]),
            client.find_object_n(None, &[]),
            client.find_object_n(None, &[]),
            client.find_object_n(None, &[]),
        )
        .unwrap();
    };

    let timeout = if cfg!(miri) {
        Duration::from_millis(1000)
    } else {
        Duration::from_millis(100)
    };

    time::timeout(timeout, deadlock).await.unwrap();

    client.shutdown();
    client_join.await.unwrap().unwrap();

    broker.shutdown().await;
    broker_join.await.unwrap();
}

#[tokio::test]
async fn find_bare_object() {
    let mut broker = TestBroker::new();
    let mut client = broker.add_client().await;

    let obj = client.create_object(ObjectUuid::new_v4()).await.unwrap();

    let id = client
        .find_bare_object(obj.id().uuid)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(id, obj.id());

    broker.join().await;
    client.join().await;
}

#[tokio::test]
async fn wait_for_bare_object() {
    let mut broker = TestBroker::new();
    let mut client = broker.add_client().await;

    let obj = client.create_object(ObjectUuid::new_v4()).await.unwrap();

    let id = client.wait_for_bare_object(obj.id().uuid).await.unwrap();
    assert_eq!(id, obj.id());

    broker.join().await;
    client.join().await;
}
