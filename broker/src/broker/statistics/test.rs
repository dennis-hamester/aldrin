use aldrin_proto::{BusListenerFilter, BusListenerScope, ObjectUuid, ServiceUuid};
use aldrin_test::tokio_based::TestBroker;

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

    // Call 2 functions.
    let reply1 = client
        .call_infallible_function::<(), ()>(svc.id(), 0, &())
        .unwrap();
    let call1 = svc.next_function_call().await.unwrap();
    let reply2 = client
        .call_infallible_function::<(), ()>(svc.id(), 0, &())
        .unwrap();
    let call2 = svc.next_function_call().await.unwrap();
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent, 2);
    assert_eq!(stats.messages_received, 2);
    assert_eq!(stats.num_function_calls, 2);
    assert_eq!(stats.functions_called, 2);
    assert_eq!(stats.functions_replied, 0);

    // Reply 1 function call.
    call1.reply.ok(&()).unwrap();
    reply1.await.unwrap();
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent, 1);
    assert_eq!(stats.messages_received, 1);
    assert_eq!(stats.num_function_calls, 1);
    assert_eq!(stats.functions_called, 0);
    assert_eq!(stats.functions_replied, 1);

    // Reply 1 function call.
    call2.reply.ok(&()).unwrap();
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
    assert_eq!(stats.events_sent, 0);

    // Emit 3 events on 0.
    client1.emit_event(svc.id(), 0, &()).unwrap();
    client1.emit_event(svc.id(), 0, &()).unwrap();
    client1.emit_event(svc.id(), 0, &()).unwrap();
    client1.sync_broker().await.unwrap();
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent, 7);
    assert_eq!(stats.messages_received, 4);
    assert_eq!(stats.events_received, 3);
    assert_eq!(stats.events_sent, 6);

    // Emit 2 events on 0.
    // Emit 1 event on 1.
    client1.emit_event(svc.id(), 0, &()).unwrap();
    client1.emit_event(svc.id(), 0, &()).unwrap();
    client1.emit_event(svc.id(), 1, &()).unwrap();
    client1.sync_broker().await.unwrap();
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent, 5);
    assert_eq!(stats.messages_received, 4);
    assert_eq!(stats.events_received, 3);
    assert_eq!(stats.events_sent, 4);

    // Final state.
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent, 0);
    assert_eq!(stats.messages_received, 0);
    assert_eq!(stats.events_received, 0);
    assert_eq!(stats.events_sent, 0);

    client1.join().await;
    client2.join().await;
    client3.join().await;
    broker.join().await;
}

#[tokio::test]
async fn channels() {
    let mut broker = TestBroker::new();
    let mut client1 = broker.add_client().await;
    let mut client2 = broker.add_client().await;

    // Initial state.
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent, 0);
    assert_eq!(stats.messages_received, 0);
    assert_eq!(stats.num_channels, 0);
    assert_eq!(stats.channels_created, 0);
    assert_eq!(stats.channels_closed, 0);
    assert_eq!(stats.items_sent, 0);

    // Create 1 channel.
    let (mut sender, _receiver) = client1
        .create_channel_with_claimed_sender::<()>()
        .await
        .unwrap();
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent, 1);
    assert_eq!(stats.messages_received, 1);
    assert_eq!(stats.num_channels, 1);
    assert_eq!(stats.channels_created, 1);
    assert_eq!(stats.channels_closed, 0);
    assert_eq!(stats.items_sent, 0);

    // Create 2 channels and close 1.
    sender.close().await.unwrap();
    let (sender1, receiver1) = client1
        .create_channel_with_claimed_sender::<()>()
        .await
        .unwrap();
    let (_sender2, mut receiver2) = client1
        .create_channel_with_claimed_receiver::<()>(1)
        .await
        .unwrap();
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent, 3);
    assert_eq!(stats.messages_received, 3);
    assert_eq!(stats.num_channels, 2);
    assert_eq!(stats.channels_created, 2);
    assert_eq!(stats.channels_closed, 1);
    assert_eq!(stats.items_sent, 0);

    // Claim 1 and send 3 items.
    let mut receiver1 = receiver1.claim(16).await.unwrap();
    let mut sender1 = sender1.established().await.unwrap();
    sender1.send_item(&()).await.unwrap();
    sender1.send_item(&()).await.unwrap();
    sender1.send_item(&()).await.unwrap();
    client1.sync_broker().await.unwrap();
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent, 6);
    assert_eq!(stats.messages_received, 5);
    assert_eq!(stats.num_channels, 2);
    assert_eq!(stats.channels_created, 0);
    assert_eq!(stats.channels_closed, 0);
    assert_eq!(stats.items_sent, 3);

    // Close 2 channels.
    sender1.close().await.unwrap();
    receiver1.close().await.unwrap();
    receiver2.close().await.unwrap();
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent, 4);
    assert_eq!(stats.messages_received, 3);
    assert_eq!(stats.num_channels, 0);
    assert_eq!(stats.channels_created, 0);
    assert_eq!(stats.channels_closed, 2);
    assert_eq!(stats.items_sent, 0);

    client1.join().await;
    client2.join().await;
    broker.join().await;
}

#[tokio::test]
async fn create_and_destroy_bus_listeners() {
    let mut broker = TestBroker::new();
    let mut client = broker.add_client().await;

    // Initial state.
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent, 0);
    assert_eq!(stats.messages_received, 0);
    assert_eq!(stats.num_bus_listeners, 0);
    assert_eq!(stats.bus_listeners_created, 0);
    assert_eq!(stats.bus_listeners_destroyed, 0);

    // Create 2 bus listeners.
    let mut bus_listener1 = client.create_bus_listener().await.unwrap();
    let mut bus_listener2 = client.create_bus_listener().await.unwrap();
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent, 2);
    assert_eq!(stats.messages_received, 2);
    assert_eq!(stats.num_bus_listeners, 2);
    assert_eq!(stats.bus_listeners_created, 2);
    assert_eq!(stats.bus_listeners_destroyed, 0);

    // Destroy 1 bus listener.
    bus_listener1.destroy().await.unwrap();
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent, 1);
    assert_eq!(stats.messages_received, 1);
    assert_eq!(stats.num_bus_listeners, 1);
    assert_eq!(stats.bus_listeners_created, 0);
    assert_eq!(stats.bus_listeners_destroyed, 1);

    // Destroy 1 bus listener.
    bus_listener2.destroy().await.unwrap();
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent, 1);
    assert_eq!(stats.messages_received, 1);
    assert_eq!(stats.num_bus_listeners, 0);
    assert_eq!(stats.bus_listeners_created, 0);
    assert_eq!(stats.bus_listeners_destroyed, 1);

    client.join().await;
    broker.join().await;
}

#[tokio::test]
async fn start_and_stop_bus_listeners() {
    let mut broker = TestBroker::new();
    let mut client = broker.add_client().await;

    let mut bus_listener1 = client.create_bus_listener().await.unwrap();
    let mut bus_listener2 = client.create_bus_listener().await.unwrap();
    broker.take_statistics().await.unwrap();

    // Initial state.
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent, 0);
    assert_eq!(stats.messages_received, 0);
    assert_eq!(stats.num_bus_listeners_active, 0);
    assert_eq!(stats.bus_listeners_started, 0);
    assert_eq!(stats.bus_listeners_stopped, 0);

    // Start 2 bus listeners.
    bus_listener1.start(BusListenerScope::All).await.unwrap();
    bus_listener2.start(BusListenerScope::All).await.unwrap();
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent, 4);
    assert_eq!(stats.messages_received, 2);
    assert_eq!(stats.num_bus_listeners_active, 2);
    assert_eq!(stats.bus_listeners_started, 2);
    assert_eq!(stats.bus_listeners_stopped, 0);

    // Stop 1 bus listener.
    bus_listener1.stop().await.unwrap();
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent, 1);
    assert_eq!(stats.messages_received, 1);
    assert_eq!(stats.num_bus_listeners_active, 1);
    assert_eq!(stats.bus_listeners_started, 0);
    assert_eq!(stats.bus_listeners_stopped, 1);

    // Stop 1 bus listener.
    bus_listener2.stop().await.unwrap();
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent, 1);
    assert_eq!(stats.messages_received, 1);
    assert_eq!(stats.num_bus_listeners_active, 0);
    assert_eq!(stats.bus_listeners_started, 0);
    assert_eq!(stats.bus_listeners_stopped, 1);

    client.join().await;
    broker.join().await;
}

#[tokio::test]
async fn bus_listener_filters() {
    let mut broker = TestBroker::new();
    let mut client = broker.add_client().await;
    let uuid = ObjectUuid::new_v4();

    let mut bus_listener = client.create_bus_listener().await.unwrap();
    broker.take_statistics().await.unwrap();

    // Initial state.
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent, 0);
    assert_eq!(stats.messages_received, 0);
    assert_eq!(stats.bus_listener_filters_added, 0);
    assert_eq!(stats.bus_listener_filters_removed, 0);
    assert_eq!(stats.bus_listener_filters_cleared, 0);

    // Add 3 filters.
    bus_listener
        .add_filter(BusListenerFilter::any_object())
        .unwrap();
    bus_listener
        .add_filter(BusListenerFilter::any_object_any_service())
        .unwrap();
    bus_listener
        .add_filter(BusListenerFilter::object(uuid))
        .unwrap();
    client.sync_broker().await.unwrap();
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent, 1);
    assert_eq!(stats.messages_received, 4);
    assert_eq!(stats.bus_listener_filters_added, 3);
    assert_eq!(stats.bus_listener_filters_removed, 0);
    assert_eq!(stats.bus_listener_filters_cleared, 0);

    // Remove 2 filters.
    bus_listener
        .remove_filter(BusListenerFilter::any_object())
        .unwrap();
    bus_listener
        .remove_filter(BusListenerFilter::any_object_any_service())
        .unwrap();
    client.sync_broker().await.unwrap();
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent, 1);
    assert_eq!(stats.messages_received, 3);
    assert_eq!(stats.bus_listener_filters_added, 0);
    assert_eq!(stats.bus_listener_filters_removed, 2);
    assert_eq!(stats.bus_listener_filters_cleared, 0);

    // Clear filters.
    bus_listener.clear_filters().unwrap();
    client.sync_broker().await.unwrap();
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent, 1);
    assert_eq!(stats.messages_received, 2);
    assert_eq!(stats.bus_listener_filters_added, 0);
    assert_eq!(stats.bus_listener_filters_removed, 0);
    assert_eq!(stats.bus_listener_filters_cleared, 1);

    client.join().await;
    broker.join().await;
}
