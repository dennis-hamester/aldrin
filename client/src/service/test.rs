use aldrin_proto::{ObjectUuid, ServiceUuid};
use aldrin_test::tokio::TestBroker;
use futures_core::stream::FusedStream;

#[tokio::test]
async fn fused_stream_terminate_after_destroy() {
    let broker = TestBroker::new();
    let client = broker.add_client().await;

    let obj = client.create_object(ObjectUuid::new_v4()).await.unwrap();
    let mut svc = obj.create_service(ServiceUuid::new_v4(), 0).await.unwrap();

    assert!(!svc.is_terminated());
    svc.destroy().await.unwrap();
    assert!(!svc.is_terminated());
    assert!(svc.next_function_call().await.is_none());
    assert!(svc.is_terminated());
}
