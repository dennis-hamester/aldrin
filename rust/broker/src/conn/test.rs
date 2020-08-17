use aldrin_client::{ObjectUuid, ServiceUuid, SubscribeMode};
use aldrin_proto::Value;
use aldrin_test::TestBroker;
use futures_util::stream::StreamExt;
use std::time::Duration;
use tokio::time;

#[tokio::test]
async fn fifo_overflow_disconnects_properly() {
    // This tests that a rather specific code path that handles fifo overflows properly informs the
    // broker about the shut down. Additionally, we must ensure that the broker itself does not send
    // any more messages to the connection, because it would detect on its own that it has shut
    // down.
    //
    // There is a high chance this test breaks in the future, such that it no longer tests the
    // specific code path, but still passes. Maybe there is a more robust way?

    // This must be an even number.
    const FIFO_SIZE: usize = 4;

    // We need to control when a connection gets polled, thus cannot use Tokio-based test framework.
    let mut broker = TestBroker::new();
    tokio::spawn(broker.take_broker().run());

    // conn1 is not spawned, but will be polled manually.
    let mut client1 = broker
        .client_builder()
        .connection_fifo_size(FIFO_SIZE)
        .build()
        .await;
    tokio::spawn(client1.take_client().run());
    let conn1 = client1.take_connection().run();
    tokio::pin!(conn1);

    // client1 creates a service, which will later be used to detect whether the broker has shut
    // down client1 properly.
    let obj = tokio::select! {
        obj = client1.create_object(ObjectUuid::new_v4()) => obj.unwrap(),
        _ = &mut conn1 => unreachable!(),
    };
    let svc = tokio::select! {
        svc = obj.create_service(ServiceUuid::new_v4()) => svc.unwrap(),
        _ = &mut conn1 => unreachable!(),
    };

    let mut client2 = broker.add_client().await;
    tokio::spawn(client2.take_client().run());
    tokio::spawn(client2.take_connection().run());

    // client1 subscribes to object events. To ensure it's actually subscribed, wait until client1
    // observes its own object created above.
    let mut objects = client1.objects(SubscribeMode::All).unwrap();
    tokio::select! {
        _ = objects.next() => {}
        _ = &mut conn1 => unreachable!(),
    }

    // This will never be replied to by client1, but the broker must abort it, when client1 is
    // disconnected.
    let reply = client2.call_function(svc.id(), 0, Value::None).unwrap();

    // client2 now causes enough events to overflow conn1's fifo.
    for _ in 0..(FIFO_SIZE / 2) {
        client2.create_object(ObjectUuid::new_v4()).await.unwrap();
    }

    // This is mildly racy, but we need to be confident that the broker has pushed everything to
    // conn1.
    time::delay_for(Duration::from_millis(100)).await;

    // At this point, there is a pending function and FIFO_SIZE-many messages in conn1's fifo, thus
    // overflowing it by exactly 1.
    assert!(time::timeout(Duration::from_millis(100), conn1)
        .await
        .unwrap()
        .is_err());

    // Again mildly racy, like above.
    time::delay_for(Duration::from_millis(100)).await;

    // Calling a function on client1's service from client2 must cause an error and not run into the
    // timeout.
    assert!(time::timeout(Duration::from_millis(100), reply)
        .await
        .unwrap()
        .is_err())
}
