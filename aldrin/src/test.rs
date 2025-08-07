use crate::core::{channel, ObjectUuid, ServiceUuid};
use crate::Client;
use aldrin_broker::Broker;
use aldrin_test::tokio::TestBroker;
use std::future::Future;
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
    let broker = TestBroker::new();
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
    let broker = TestBroker::new();
    let client = broker.add_client().await;

    let obj = client.create_object(ObjectUuid::new_v4()).await.unwrap();
    let uuid = ServiceUuid::new_v4();
    let fut = obj.create_service(uuid, 0);

    // This assumes that polling the future once is enough to create the service.
    PollOnce(Box::pin(fut)).await;

    // The service may have been created temporarily. Give client and broker some time to destroy it
    // again.
    time::sleep(Duration::from_millis(100)).await;

    assert!(obj.create_service(uuid, 0).await.is_ok());
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
            client.find_object(None, &[]),
            client.find_object(None, &[]),
            client.find_object(None, &[]),
            client.find_object(None, &[]),
        )
        .unwrap();
    };

    time::timeout(Duration::from_millis(100), deadlock)
        .await
        .unwrap();

    client.shutdown();
    client_join.await.unwrap().unwrap();

    broker.shutdown().await;
    broker_join.await.unwrap();
}
