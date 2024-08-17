use crate::core::{ObjectUuid, ServiceUuid, TypeId};
use aldrin_test::aldrin::low_level::ServiceInfo;
use aldrin_test::aldrin::Error;
use aldrin_test::tokio::TestBroker;
use futures_core::stream::FusedStream;
use std::time::Duration;
use tokio::time;
use uuid::uuid;

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

#[tokio::test]
async fn proxy_getter() {
    const TYPE_ID: TypeId = TypeId(uuid!("e6cffd81-51fb-4466-ac58-758db91d6bfa"));

    let broker = TestBroker::new();
    let client = broker.add_client().await;

    let obj = client.create_object(ObjectUuid::new_v4()).await.unwrap();
    let info = ServiceInfo::new(1).set_type_id(TYPE_ID);
    let svc = obj
        .create_service(ServiceUuid::new_v4(), info)
        .await
        .unwrap();

    let proxy = client.create_proxy(svc.id()).await.unwrap();

    assert_eq!(proxy.id(), svc.id());
    assert_eq!(proxy.version(), 1);
    assert_eq!(proxy.type_id(), Some(TYPE_ID));
}

#[tokio::test]
async fn call_ok() {
    let broker = TestBroker::new();
    let client = broker.add_client().await;

    let obj = client.create_object(ObjectUuid::new_v4()).await.unwrap();
    let info = ServiceInfo::new(0);
    let mut svc = obj
        .create_service(ServiceUuid::new_v4(), info)
        .await
        .unwrap();

    let proxy = client.create_proxy(svc.id()).await.unwrap();
    let reply = proxy.call(0, &());

    let call = svc.next_call().await.unwrap();
    assert_eq!(call.deserialize(), Ok(()));
    call.into_promise().ok(&()).unwrap();

    let reply = reply.await.unwrap();
    assert_eq!(reply.unwrap().deserialize(), Ok(()));
}

#[tokio::test]
async fn call_done() {
    let broker = TestBroker::new();
    let client = broker.add_client().await;

    let obj = client.create_object(ObjectUuid::new_v4()).await.unwrap();
    let info = ServiceInfo::new(0);
    let mut svc = obj
        .create_service(ServiceUuid::new_v4(), info)
        .await
        .unwrap();

    let proxy = client.create_proxy(svc.id()).await.unwrap();
    let reply = proxy.call(0, &());

    let call = svc.next_call().await.unwrap();
    assert_eq!(call.deserialize(), Ok(()));
    call.into_promise().done().unwrap();

    let reply = reply.await.unwrap();
    assert_eq!(reply.unwrap().deserialize(), Ok(()));
}

#[tokio::test]
async fn call_err() {
    let broker = TestBroker::new();
    let client = broker.add_client().await;

    let obj = client.create_object(ObjectUuid::new_v4()).await.unwrap();
    let info = ServiceInfo::new(0);
    let mut svc = obj
        .create_service(ServiceUuid::new_v4(), info)
        .await
        .unwrap();

    let proxy = client.create_proxy(svc.id()).await.unwrap();
    let reply = proxy.call(0, &());

    let call = svc.next_call().await.unwrap();
    assert_eq!(call.deserialize(), Ok(()));
    call.into_promise().err(&()).unwrap();

    let reply = reply.await.unwrap();
    assert_eq!(reply.unwrap_err().deserialize(), Ok(()));
}

#[tokio::test]
async fn call_abort_by_callee() {
    let broker = TestBroker::new();
    let client = broker.add_client().await;

    let obj = client.create_object(ObjectUuid::new_v4()).await.unwrap();
    let info = ServiceInfo::new(0);
    let mut svc = obj
        .create_service(ServiceUuid::new_v4(), info)
        .await
        .unwrap();

    let proxy = client.create_proxy(svc.id()).await.unwrap();
    let reply = proxy.call(0, &());

    let call = svc.next_call().await.unwrap();
    assert_eq!(call.deserialize(), Ok(()));
    call.into_promise().abort().unwrap();

    assert_eq!(reply.await, Err(Error::CallAborted));
}

#[tokio::test]
async fn call_abort_by_caller() {
    let broker = TestBroker::new();
    let client = broker.add_client().await;

    let obj = client.create_object(ObjectUuid::new_v4()).await.unwrap();
    let info = ServiceInfo::new(0);
    let mut svc = obj
        .create_service(ServiceUuid::new_v4(), info)
        .await
        .unwrap();

    let proxy = client.create_proxy(svc.id()).await.unwrap();
    let reply = proxy.call(0, &());
    reply.abort();

    let call = svc.next_call().await.unwrap();
    assert_eq!(call.deserialize(), Ok(()));

    let mut promise = call.into_promise();
    promise.aborted().await;
    assert!(promise.is_aborted());
}

#[tokio::test]
async fn call_invalid_function() {
    let broker = TestBroker::new();
    let client = broker.add_client().await;

    let obj = client.create_object(ObjectUuid::new_v4()).await.unwrap();
    let info = ServiceInfo::new(0);
    let mut svc = obj
        .create_service(ServiceUuid::new_v4(), info)
        .await
        .unwrap();

    let proxy = client.create_proxy(svc.id()).await.unwrap();
    let reply = proxy.call(0, &());

    let call = svc.next_call().await.unwrap();
    assert_eq!(call.deserialize(), Ok(()));
    call.into_promise().invalid_function().unwrap();

    assert_eq!(reply.await, Err(Error::invalid_function(0)));
}

#[tokio::test]
async fn call_invalid_args() {
    let broker = TestBroker::new();
    let client = broker.add_client().await;

    let obj = client.create_object(ObjectUuid::new_v4()).await.unwrap();
    let info = ServiceInfo::new(0);
    let mut svc = obj
        .create_service(ServiceUuid::new_v4(), info)
        .await
        .unwrap();

    let proxy = client.create_proxy(svc.id()).await.unwrap();
    let reply = proxy.call(0, &());

    let call = svc.next_call().await.unwrap();
    assert_eq!(call.deserialize(), Ok(()));
    call.into_promise().invalid_args().unwrap();

    assert_eq!(reply.await, Err(Error::invalid_arguments(0, None)));
}

#[tokio::test]
async fn subscribe_event() {
    let broker = TestBroker::new();
    let client = broker.add_client().await;

    let obj = client.create_object(ObjectUuid::new_v4()).await.unwrap();
    let info = ServiceInfo::new(0);
    let svc = obj
        .create_service(ServiceUuid::new_v4(), info)
        .await
        .unwrap();

    let mut proxy = client.create_proxy(svc.id()).await.unwrap();
    proxy.subscribe(0).await.unwrap();

    svc.emit(1, &()).unwrap();
    svc.emit(0, &()).unwrap();

    let ev = proxy.next_event().await.unwrap();
    assert_eq!(ev.id(), 0);
    assert_eq!(ev.deserialize(), Ok(()));
}

#[tokio::test]
async fn unsubscribe_event() {
    let broker = TestBroker::new();
    let client = broker.add_client().await;

    let obj = client.create_object(ObjectUuid::new_v4()).await.unwrap();
    let info = ServiceInfo::new(0);
    let svc = obj
        .create_service(ServiceUuid::new_v4(), info)
        .await
        .unwrap();

    let mut proxy = client.create_proxy(svc.id()).await.unwrap();
    proxy.subscribe(0).await.unwrap();
    proxy.subscribe(1).await.unwrap();

    svc.emit(0, &()).unwrap();
    svc.emit(1, &()).unwrap();

    let ev = proxy.next_event().await.unwrap();
    assert_eq!(ev.id(), 0);
    assert_eq!(ev.deserialize(), Ok(()));

    let ev = proxy.next_event().await.unwrap();
    assert_eq!(ev.id(), 1);
    assert_eq!(ev.deserialize(), Ok(()));

    proxy.unsubscribe(0).await.unwrap();
    client.sync_broker().await.unwrap();

    svc.emit(0, &()).unwrap();
    svc.emit(1, &()).unwrap();

    let ev = proxy.next_event().await.unwrap();
    assert_eq!(ev.id(), 1);
    assert_eq!(ev.deserialize(), Ok(()));
}

#[tokio::test]
async fn events_mutliple_proxies() {
    let broker = TestBroker::new();
    let client = broker.add_client().await;

    let obj = client.create_object(ObjectUuid::new_v4()).await.unwrap();
    let info = ServiceInfo::new(0);
    let svc = obj
        .create_service(ServiceUuid::new_v4(), info)
        .await
        .unwrap();

    let mut proxy1 = client.create_proxy(svc.id()).await.unwrap();
    proxy1.subscribe(0).await.unwrap();
    proxy1.subscribe(1).await.unwrap();

    let mut proxy2 = client.create_proxy(svc.id()).await.unwrap();
    proxy2.subscribe(0).await.unwrap();
    proxy2.subscribe(1).await.unwrap();

    svc.emit(0, &()).unwrap();

    let ev = proxy1.next_event().await.unwrap();
    assert_eq!(ev.id(), 0);
    assert_eq!(ev.deserialize(), Ok(()));

    let ev = proxy2.next_event().await.unwrap();
    assert_eq!(ev.id(), 0);
    assert_eq!(ev.deserialize(), Ok(()));

    proxy2.unsubscribe(0).await.unwrap();
    client.sync_broker().await.unwrap();

    svc.emit(0, &()).unwrap();
    svc.emit(1, &()).unwrap();

    let ev = proxy1.next_event().await.unwrap();
    assert_eq!(ev.id(), 0);
    assert_eq!(ev.deserialize(), Ok(()));

    let ev = proxy1.next_event().await.unwrap();
    assert_eq!(ev.id(), 1);
    assert_eq!(ev.deserialize(), Ok(()));

    let ev = proxy2.next_event().await.unwrap();
    assert_eq!(ev.id(), 1);
    assert_eq!(ev.deserialize(), Ok(()));
}

#[tokio::test]
async fn no_unnecessary_events_emitted() {
    let mut broker = TestBroker::new();
    let client = broker.add_client().await;

    let obj = client.create_object(ObjectUuid::new_v4()).await.unwrap();
    let info = ServiceInfo::new(0);
    let svc = obj
        .create_service(ServiceUuid::new_v4(), info)
        .await
        .unwrap();

    let proxy = client.create_proxy(svc.id()).await.unwrap();
    proxy.subscribe(0).await.unwrap();
    client.sync_broker().await.unwrap();
    broker.take_statistics().await.unwrap();

    svc.emit(0, &()).unwrap();
    client.sync_broker().await.unwrap();
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_received(), 2);

    proxy.unsubscribe(0).await.unwrap();
    client.sync_broker().await.unwrap();
    broker.take_statistics().await.unwrap();

    svc.emit(0, &()).unwrap();
    client.sync_broker().await.unwrap();
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_received(), 1);
}

#[tokio::test]
async fn close_events_with_subscribers() {
    let broker = TestBroker::new();
    let client = broker.add_client().await;

    let obj = client.create_object(ObjectUuid::new_v4()).await.unwrap();
    let info = ServiceInfo::new(0);
    let svc = obj
        .create_service(ServiceUuid::new_v4(), info)
        .await
        .unwrap();

    let mut proxy = client.create_proxy(svc.id()).await.unwrap();
    proxy.subscribe(0).await.unwrap();
    svc.destroy().await.unwrap();

    assert!(proxy.next_event().await.is_none());
}

#[tokio::test]
async fn close_events_without_subscribers() {
    let broker = TestBroker::new();
    let client = broker.add_client().await;

    let obj = client.create_object(ObjectUuid::new_v4()).await.unwrap();
    let info = ServiceInfo::new(0);
    let svc = obj
        .create_service(ServiceUuid::new_v4(), info)
        .await
        .unwrap();

    let mut proxy = client.create_proxy(svc.id()).await.unwrap();
    svc.destroy().await.unwrap();

    let event = time::timeout(Duration::from_millis(100), proxy.next_event())
        .await
        .unwrap();

    assert!(event.is_none());
}

#[tokio::test]
async fn subscribe_multiple_services_same_event_id() {
    let broker = TestBroker::new();
    let client = broker.add_client().await;

    let obj = client.create_object(ObjectUuid::new_v4()).await.unwrap();
    let info = ServiceInfo::new(0);

    let svc1 = obj
        .create_service(ServiceUuid::new_v4(), info)
        .await
        .unwrap();
    let mut proxy1 = client.create_proxy(svc1.id()).await.unwrap();

    let svc2 = obj
        .create_service(ServiceUuid::new_v4(), info)
        .await
        .unwrap();
    let mut proxy2 = client.create_proxy(svc2.id()).await.unwrap();

    proxy1.subscribe(0).await.unwrap();
    proxy2.subscribe(0).await.unwrap();

    svc1.emit(0, &1).unwrap();
    svc2.emit(0, &2).unwrap();

    let event1 = time::timeout(Duration::from_millis(100), proxy1.next_event())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(event1.id(), 0);
    assert_eq!(event1.deserialize(), Ok(1));

    let event2 = time::timeout(Duration::from_millis(100), proxy2.next_event())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(event2.id(), 0);
    assert_eq!(event2.deserialize(), Ok(2));

    proxy2.unsubscribe(0).await.unwrap();
    client.sync_broker().await.unwrap();

    svc2.emit(0, &3).unwrap();
    assert!(
        time::timeout(Duration::from_millis(100), proxy2.next_event())
            .await
            .is_err()
    );
}

#[tokio::test]
async fn can_subscribe_all() {
    let broker = TestBroker::new();
    let client = broker.add_client().await;

    let obj = client.create_object(ObjectUuid::new_v4()).await.unwrap();

    let svc = obj
        .create_service(ServiceUuid::new_v4(), ServiceInfo::new(0))
        .await
        .unwrap();
    let proxy = client.create_proxy(svc.id()).await.unwrap();

    assert!(proxy.can_subscribe_all());
}
