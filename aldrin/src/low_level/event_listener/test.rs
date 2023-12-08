use crate::core::{ObjectUuid, ServiceUuid};
use aldrin_test::tokio::TestBroker;
use futures_util::stream::FusedStream;
use std::time::Duration;
use tokio::time;

#[tokio::test]
async fn stop_on_client_shutdown() {
    let broker = TestBroker::new();
    let mut client = broker.add_client().await;

    let obj = client.create_object(ObjectUuid::new_v4()).await.unwrap();
    let svc = obj.create_service(ServiceUuid::new_v4(), 0).await.unwrap();

    let mut events = client.create_event_listener();
    events.subscribe(svc.id(), 0).await.unwrap();

    client.shutdown();
    client.join().await;

    let event = time::timeout(Duration::from_millis(100), events.next_event())
        .await
        .unwrap();
    assert!(event.is_none());
    assert!(events.is_terminated());
}

#[tokio::test]
async fn stop_on_broker_shutdown() {
    let mut broker = TestBroker::new();
    let mut client = broker.add_client().await;

    let obj = client.create_object(ObjectUuid::new_v4()).await.unwrap();
    let svc = obj.create_service(ServiceUuid::new_v4(), 0).await.unwrap();

    let mut events = client.create_event_listener();
    events.subscribe(svc.id(), 0).await.unwrap();

    broker.shutdown().await;
    client.join().await;

    let event = time::timeout(Duration::from_millis(100), events.next_event())
        .await
        .unwrap();
    assert!(event.is_none());
    assert!(events.is_terminated());
}
