use crate::test::TestBroker;
use aldrin_client::{ObjectUuid, ServiceUuid};
use aldrin_proto::Value;
use futures_util::stream::StreamExt;
use uuid::Uuid;

#[tokio::test]
async fn disconnect_during_function_call() {
    let broker = TestBroker::new();

    let client1 = broker.add_client().await;
    let obj = client1.create_object(ObjectUuid::new_v4()).await.unwrap();
    let mut svc = obj
        .create_service(ServiceUuid(Uuid::new_v4()))
        .await
        .unwrap();

    // client2 calls a function on client1 and disconnects before client1 replies.
    let client2 = broker.add_client().await;
    let _ = client2.call_function(svc.id(), 0, Value::None).unwrap();
    client2.shutdown();
    client2.join().await;

    let call = svc.next().await.unwrap();
    call.reply.ok(Value::None).unwrap();
    client1.shutdown();

    broker.join_idle().await
}
