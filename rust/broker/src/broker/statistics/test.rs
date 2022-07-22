use aldrin_client::{ObjectUuid, ServiceUuid};
use aldrin_test::tokio_based::TestBroker;
use futures_util::stream::StreamExt;
use std::time::Duration;
use tokio::time;

#[tokio::test]
async fn timestamp_monotonicity() {
    let mut broker = TestBroker::new();

    let stats1 = broker.take_statistics().await.unwrap();
    assert!(stats1.end > stats1.start);

    let stats2 = broker.take_statistics().await.unwrap();
    assert!(stats2.end > stats2.start);
    assert_eq!(stats2.start, stats1.end);

    let stats3 = broker.take_statistics().await.unwrap();
    assert!(stats3.end > stats3.start);
    assert_eq!(stats3.start, stats2.end);

    broker.join().await;
}

#[tokio::test]
async fn connections() {
    let mut broker = TestBroker::new();

    // Initial state.
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent, 0);
    assert_eq!(stats.messages_received, 0);
    assert_eq!(stats.num_connections, 0);
    assert_eq!(stats.connections_added, 0);
    assert_eq!(stats.connections_shut_down, 0);

    // Add 1 client.
    let mut client1 = broker.add_client().await;
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent, 0);
    assert_eq!(stats.messages_received, 0);
    assert_eq!(stats.num_connections, 1);
    assert_eq!(stats.connections_added, 1);
    assert_eq!(stats.connections_shut_down, 0);

    // Remove 1 client and add 2.
    client1.join().await;
    let mut client2 = broker.add_client().await;
    let mut client3 = broker.add_client().await;
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent, 1);
    assert_eq!(stats.messages_received, 0);
    assert_eq!(stats.num_connections, 2);
    assert_eq!(stats.connections_added, 2);
    assert_eq!(stats.connections_shut_down, 1);

    // Remove 2 clients.
    client2.join().await;
    client3.join().await;
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent, 2);
    assert_eq!(stats.messages_received, 0);
    assert_eq!(stats.num_connections, 0);
    assert_eq!(stats.connections_added, 0);
    assert_eq!(stats.connections_shut_down, 2);

    // Final state.
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent, 0);
    assert_eq!(stats.messages_received, 0);
    assert_eq!(stats.num_connections, 0);
    assert_eq!(stats.connections_added, 0);
    assert_eq!(stats.connections_shut_down, 0);

    broker.join().await;
}

#[tokio::test]
async fn objects() {
    let mut broker = TestBroker::new();
    let mut client = broker.add_client().await;

    // Initial state.
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent, 0);
    assert_eq!(stats.messages_received, 0);
    assert_eq!(stats.num_objects, 0);
    assert_eq!(stats.objects_created, 0);
    assert_eq!(stats.objects_destroyed, 0);

    // Create 1 object.
    let obj1 = client.create_object(ObjectUuid::new_v4()).await.unwrap();
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent, 1);
    assert_eq!(stats.messages_received, 1);
    assert_eq!(stats.num_objects, 1);
    assert_eq!(stats.objects_created, 1);
    assert_eq!(stats.objects_destroyed, 0);

    // Destroy 1 object and create 2.
    obj1.destroy().await.unwrap();
    let obj2 = client.create_object(ObjectUuid::new_v4()).await.unwrap();
    let obj3 = client.create_object(ObjectUuid::new_v4()).await.unwrap();
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent, 3);
    assert_eq!(stats.messages_received, 3);
    assert_eq!(stats.num_objects, 2);
    assert_eq!(stats.objects_created, 2);
    assert_eq!(stats.objects_destroyed, 1);

    // Destroy 2 objects.
    obj2.destroy().await.unwrap();
    obj3.destroy().await.unwrap();
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent, 2);
    assert_eq!(stats.messages_received, 2);
    assert_eq!(stats.num_objects, 0);
    assert_eq!(stats.objects_created, 0);
    assert_eq!(stats.objects_destroyed, 2);

    // Final state.
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent, 0);
    assert_eq!(stats.messages_received, 0);
    assert_eq!(stats.num_objects, 0);
    assert_eq!(stats.objects_created, 0);
    assert_eq!(stats.objects_destroyed, 0);

    client.join().await;
    broker.join().await;
}

#[tokio::test]
async fn services() {
    let mut broker = TestBroker::new();
    let mut client = broker.add_client().await;

    // Initial state.
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent, 0);
    assert_eq!(stats.messages_received, 0);
    assert_eq!(stats.num_services, 0);
    assert_eq!(stats.services_created, 0);
    assert_eq!(stats.services_destroyed, 0);

    // Create 1 object with 3 services.
    let obj = client.create_object(ObjectUuid::new_v4()).await.unwrap();
    let svc1 = obj.create_service(ServiceUuid::new_v4(), 0).await.unwrap();
    let svc2 = obj.create_service(ServiceUuid::new_v4(), 0).await.unwrap();
    let svc3 = obj.create_service(ServiceUuid::new_v4(), 0).await.unwrap();
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent, 4);
    assert_eq!(stats.messages_received, 4);
    assert_eq!(stats.num_services, 3);
    assert_eq!(stats.services_created, 3);
    assert_eq!(stats.services_destroyed, 0);

    // Destroy 1 service.
    svc1.destroy().await.unwrap();
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent, 1);
    assert_eq!(stats.messages_received, 1);
    assert_eq!(stats.num_services, 2);
    assert_eq!(stats.services_created, 0);
    assert_eq!(stats.services_destroyed, 1);

    // Destroy 2 services.
    svc2.destroy().await.unwrap();
    svc3.destroy().await.unwrap();
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent, 2);
    assert_eq!(stats.messages_received, 2);
    assert_eq!(stats.num_services, 0);
    assert_eq!(stats.services_created, 0);
    assert_eq!(stats.services_destroyed, 2);

    // Final state.
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent, 0);
    assert_eq!(stats.messages_received, 0);
    assert_eq!(stats.num_services, 0);
    assert_eq!(stats.services_created, 0);
    assert_eq!(stats.services_destroyed, 0);

    client.join().await;
    broker.join().await;
}

#[tokio::test]
async fn function_calls() {
    let mut broker = TestBroker::new();
    let mut client = broker.add_client().await;
    let obj = client.create_object(ObjectUuid::new_v4()).await.unwrap();
    let mut svc = obj.create_service(ServiceUuid::new_v4(), 0).await.unwrap();

    // Initial state.
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent, 2);
    assert_eq!(stats.messages_received, 2);
    assert_eq!(stats.num_function_calls, 0);
    assert_eq!(stats.functions_called, 0);
    assert_eq!(stats.functions_replied, 0);

    // Call 2 function.
    let reply1 = client
        .call_infallible_function::<(), ()>(svc.id(), 0, ())
        .unwrap();
    let call1 = svc.next().await.unwrap();
    let reply2 = client
        .call_infallible_function::<(), ()>(svc.id(), 0, ())
        .unwrap();
    let call2 = svc.next().await.unwrap();
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent, 2);
    assert_eq!(stats.messages_received, 2);
    assert_eq!(stats.num_function_calls, 2);
    assert_eq!(stats.functions_called, 2);
    assert_eq!(stats.functions_replied, 0);

    // Reply 1 function call.
    call1.reply.ok(()).unwrap();
    reply1.await.unwrap();
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent, 1);
    assert_eq!(stats.messages_received, 1);
    assert_eq!(stats.num_function_calls, 1);
    assert_eq!(stats.functions_called, 0);
    assert_eq!(stats.functions_replied, 1);

    // Reply 1 function call.
    call2.reply.ok(()).unwrap();
    reply2.await.unwrap();
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent, 1);
    assert_eq!(stats.messages_received, 1);
    assert_eq!(stats.num_function_calls, 0);
    assert_eq!(stats.functions_called, 0);
    assert_eq!(stats.functions_replied, 1);

    // Final state.
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent, 0);
    assert_eq!(stats.messages_received, 0);
    assert_eq!(stats.num_function_calls, 0);
    assert_eq!(stats.functions_called, 0);
    assert_eq!(stats.functions_replied, 0);

    client.join().await;
    broker.join().await;
}

#[tokio::test]
async fn events() {
    let mut broker = TestBroker::new();

    let mut client1 = broker.add_client().await;
    let obj = client1.create_object(ObjectUuid::new_v4()).await.unwrap();
    let svc = obj.create_service(ServiceUuid::new_v4(), 0).await.unwrap();

    let mut client2 = broker.add_client().await;
    let mut events2 = client2.events();
    events2.subscribe(svc.id(), 0).await.unwrap();

    let mut client3 = broker.add_client().await;
    let mut events3 = client3.events();
    events3.subscribe(svc.id(), 0).await.unwrap();

    // Initial state.
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent, 5);
    assert_eq!(stats.messages_received, 4);
    assert_eq!(stats.events_received, 0);

    // Emit 3 events on 0.
    client1.emit_event(svc.id(), 0, ()).unwrap();
    client1.emit_event(svc.id(), 0, ()).unwrap();
    client1.emit_event(svc.id(), 0, ()).unwrap();
    // HACK: Aldrin doesn't have a proper way of synchronizing with the broker.
    time::sleep(Duration::from_millis(100)).await;
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent, 6);
    assert_eq!(stats.messages_received, 3);
    assert_eq!(stats.events_received, 3);

    // Emit 2 events on 0.
    // Emit 1 event on 1.
    client1.emit_event(svc.id(), 0, ()).unwrap();
    client1.emit_event(svc.id(), 0, ()).unwrap();
    client1.emit_event(svc.id(), 1, ()).unwrap();
    // HACK: Aldrin doesn't have a proper way of synchronizing with the broker.
    time::sleep(Duration::from_millis(100)).await;
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent, 4);
    assert_eq!(stats.messages_received, 3);
    assert_eq!(stats.events_received, 3);

    // Final state.
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent, 0);
    assert_eq!(stats.messages_received, 0);
    assert_eq!(stats.events_received, 0);

    client1.join().await;
    client2.join().await;
    client3.join().await;
    broker.join().await;
}
