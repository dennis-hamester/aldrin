use aldrin_test::aldrin_client::{ObjectUuid, ServiceUuid};
use aldrin_test::tokio_based::TestBroker;
use futures_core::stream::FusedStream;
use futures_util::stream::StreamExt;

#[tokio::test]
async fn fused_stream_terminate_after_destroy() {
    let broker = TestBroker::new();
    let client = broker.add_client().await;

    let obj = client.create_object(ObjectUuid::new_v4()).await.unwrap();
    let mut svc = obj.create_service(ServiceUuid::new_v4()).await.unwrap();

    assert!(!svc.is_terminated());
    svc.destroy().await.unwrap();
    assert!(svc.next().await.is_none());
    assert!(svc.is_terminated());
}

#[tokio::test]
async fn no_panic_after_none() {
    let broker = TestBroker::new();
    let client = broker.add_client().await;

    let obj = client.create_object(ObjectUuid::new_v4()).await.unwrap();
    let mut svc = obj.create_service(ServiceUuid::new_v4()).await.unwrap();

    svc.destroy().await.unwrap();
    assert!(svc.next().await.is_none());
    assert!(svc.next().await.is_none());
}
