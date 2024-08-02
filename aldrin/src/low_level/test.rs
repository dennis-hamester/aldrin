use crate::core::{ObjectUuid, ServiceInfo, ServiceUuid};
use aldrin_test::tokio::TestBroker;
use futures_core::stream::FusedStream;
use std::time::Duration;
use tokio::time;

#[tokio::test]
async fn stop_events_on_client_shutdown() {
    let broker = TestBroker::new();
    let mut client = broker.add_client().await;

    let obj = client.create_object(ObjectUuid::new_v4()).await.unwrap();
    let info = ServiceInfo::new(0);
    let svc = obj
        .create_service(ServiceUuid::new_v4(), info)
        .await
        .unwrap();

    let mut proxy = client.create_proxy(svc.id()).await.unwrap();
    proxy.subscribe(0).await.unwrap();

    client.shutdown();
    client.join().await;

    let event = time::timeout(Duration::from_millis(100), proxy.next_event())
        .await
        .unwrap();
    assert!(event.is_none());
    assert!(proxy.events_finished());
}

#[tokio::test]
async fn stop_event_on_broker_shutdown() {
    let mut broker = TestBroker::new();
    let mut client = broker.add_client().await;

    let obj = client.create_object(ObjectUuid::new_v4()).await.unwrap();
    let info = ServiceInfo::new(0);
    let svc = obj
        .create_service(ServiceUuid::new_v4(), info)
        .await
        .unwrap();

    let mut proxy = client.create_proxy(svc.id()).await.unwrap();
    proxy.subscribe(0).await.unwrap();

    broker.shutdown().await;
    client.join().await;

    let event = time::timeout(Duration::from_millis(100), proxy.next_event())
        .await
        .unwrap();
    assert!(event.is_none());
    assert!(proxy.events_finished());
}

#[tokio::test]
async fn fused_stream_terminate_after_destroy() {
    let broker = TestBroker::new();
    let client = broker.add_client().await;

    let obj = client.create_object(ObjectUuid::new_v4()).await.unwrap();
    let info = ServiceInfo::new(0);
    let mut svc = obj
        .create_service(ServiceUuid::new_v4(), info)
        .await
        .unwrap();

    assert!(!svc.is_terminated());
    svc.destroy().await.unwrap();
    assert!(!svc.is_terminated());
    assert!(svc.next_call().await.is_none());
    assert!(svc.is_terminated());
}
