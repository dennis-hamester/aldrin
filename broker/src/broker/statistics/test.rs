use crate::core::{ObjectUuid, ServiceUuid};
use aldrin::low_level::{Proxy, ServiceInfo};
use aldrin_test::tokio::TestBroker;

#[tokio::test]
async fn timestamp_monotonicity() {
    let mut broker = TestBroker::new();

    let stats1 = broker.take_statistics().await.unwrap();
    assert!(stats1.end() > stats1.start());

    let stats2 = broker.take_statistics().await.unwrap();
    assert!(stats2.end() > stats2.start());
    assert_eq!(stats2.start(), stats1.end());

    let stats3 = broker.take_statistics().await.unwrap();
    assert!(stats3.end() > stats3.start());
    assert_eq!(stats3.start(), stats2.end());

    broker.join().await;
}

#[tokio::test]
async fn connections() {
    let mut broker = TestBroker::new();

    // Initial state.
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent(), 0);
    assert_eq!(stats.messages_received(), 0);
    assert_eq!(stats.num_connections(), 0);

    // Add 1 client.
    let mut client1 = broker.add_client().await;
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent(), 0);
    assert_eq!(stats.messages_received(), 0);
    assert_eq!(stats.num_connections(), 1);

    // Remove 1 client and add 2.
    client1.join().await;
    let mut client2 = broker.add_client().await;
    let mut client3 = broker.add_client().await;
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent(), 0);
    assert_eq!(stats.messages_received(), 0);
    assert_eq!(stats.num_connections(), 2);

    // Remove 2 clients.
    client2.join().await;
    client3.join().await;
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent(), 0);
    assert_eq!(stats.messages_received(), 0);
    assert_eq!(stats.num_connections(), 0);

    // Final state.
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent(), 0);
    assert_eq!(stats.messages_received(), 0);
    assert_eq!(stats.num_connections(), 0);

    broker.join().await;
}

#[tokio::test]
async fn objects() {
    let mut broker = TestBroker::new();
    let mut client = broker.add_client().await;

    // Initial state.
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent(), 0);
    assert_eq!(stats.messages_received(), 0);
    assert_eq!(stats.num_objects(), 0);

    // Create 1 object.
    let obj1 = client.create_object(ObjectUuid::new_v4()).await.unwrap();
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent(), 1);
    assert_eq!(stats.messages_received(), 1);
    assert_eq!(stats.num_objects(), 1);

    // Destroy 1 object and create 2.
    obj1.destroy().await.unwrap();
    let obj2 = client.create_object(ObjectUuid::new_v4()).await.unwrap();
    let obj3 = client.create_object(ObjectUuid::new_v4()).await.unwrap();
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent(), 3);
    assert_eq!(stats.messages_received(), 3);
    assert_eq!(stats.num_objects(), 2);

    // Destroy 2 objects.
    obj2.destroy().await.unwrap();
    obj3.destroy().await.unwrap();
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent(), 2);
    assert_eq!(stats.messages_received(), 2);
    assert_eq!(stats.num_objects(), 0);

    // Final state.
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent(), 0);
    assert_eq!(stats.messages_received(), 0);
    assert_eq!(stats.num_objects(), 0);

    client.join().await;
    broker.join().await;
}

#[tokio::test]
async fn services() {
    let mut broker = TestBroker::new();
    let mut client = broker.add_client().await;

    // Initial state.
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent(), 0);
    assert_eq!(stats.messages_received(), 0);
    assert_eq!(stats.num_services(), 0);

    // Create 1 object with 3 services.
    let obj = client.create_object(ObjectUuid::new_v4()).await.unwrap();
    let info = ServiceInfo::new(0);
    let svc1 = obj
        .create_service(ServiceUuid::new_v4(), info)
        .await
        .unwrap();
    let svc2 = obj
        .create_service(ServiceUuid::new_v4(), info)
        .await
        .unwrap();
    let svc3 = obj
        .create_service(ServiceUuid::new_v4(), info)
        .await
        .unwrap();
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent(), 4);
    assert_eq!(stats.messages_received(), 4);
    assert_eq!(stats.num_services(), 3);

    // Destroy 1 service.
    svc1.destroy().await.unwrap();
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent(), 1);
    assert_eq!(stats.messages_received(), 1);
    assert_eq!(stats.num_services(), 2);

    // Destroy 2 services.
    svc2.destroy().await.unwrap();
    svc3.destroy().await.unwrap();
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent(), 2);
    assert_eq!(stats.messages_received(), 2);
    assert_eq!(stats.num_services(), 0);

    // Final state.
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent(), 0);
    assert_eq!(stats.messages_received(), 0);
    assert_eq!(stats.num_services(), 0);

    client.join().await;
    broker.join().await;
}

#[tokio::test]
async fn function_calls() {
    let mut broker = TestBroker::new();
    let mut client = broker.add_client().await;
    let obj = client.create_object(ObjectUuid::new_v4()).await.unwrap();
    let info = ServiceInfo::new(0);
    let mut svc = obj
        .create_service(ServiceUuid::new_v4(), info)
        .await
        .unwrap();
    let proxy = Proxy::new(&client, svc.id()).await.unwrap();

    // Initial state.
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent(), 4);
    assert_eq!(stats.messages_received(), 4);

    // Call 2 functions.
    let reply1 = proxy.call(0, &(), None);
    let call1 = svc.next_call().await.unwrap();
    let reply2 = proxy.call(0, &(), None);
    let call2 = svc.next_call().await.unwrap();
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent(), 2);
    assert_eq!(stats.messages_received(), 2);

    // Reply 1 function call.
    call1.into_promise().ok(&()).unwrap();
    reply1.await.unwrap().into_args().unwrap();
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent(), 1);
    assert_eq!(stats.messages_received(), 1);

    // Reply 1 function call.
    call2.into_promise().ok(&()).unwrap();
    reply2.await.unwrap().into_args().unwrap();
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent(), 1);
    assert_eq!(stats.messages_received(), 1);

    // Final state.
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent(), 0);
    assert_eq!(stats.messages_received(), 0);

    client.join().await;
    broker.join().await;
}

#[tokio::test]
async fn events() {
    let mut broker = TestBroker::new();

    let mut client1 = broker.add_client().await;
    let obj = client1.create_object(ObjectUuid::new_v4()).await.unwrap();
    let info = ServiceInfo::new(0);
    let svc = obj
        .create_service(ServiceUuid::new_v4(), info)
        .await
        .unwrap();

    let mut client2 = broker.add_client().await;
    let proxy2 = client2.create_proxy(svc.id()).await.unwrap();
    proxy2.subscribe(0).await.unwrap();

    let mut client3 = broker.add_client().await;
    let proxy3 = client3.create_proxy(svc.id()).await.unwrap();
    proxy3.subscribe(0).await.unwrap();

    // Initial state.
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent(), 9);
    assert_eq!(stats.messages_received(), 8);

    // Emit 3 events on 0.
    svc.emit(0, &()).unwrap();
    svc.emit(0, &()).unwrap();
    svc.emit(0, &()).unwrap();
    client1.sync_broker().await.unwrap();
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent(), 7);
    assert_eq!(stats.messages_received(), 4);

    // Emit 2 events on 0.
    // Emit 1 event on 1.
    svc.emit(0, &()).unwrap();
    svc.emit(0, &()).unwrap();
    svc.emit(1, &()).unwrap();
    client1.sync_broker().await.unwrap();
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent(), 5);
    assert_eq!(stats.messages_received(), 3);

    // Final state.
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent(), 0);
    assert_eq!(stats.messages_received(), 0);

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
    assert_eq!(stats.messages_sent(), 0);
    assert_eq!(stats.messages_received(), 0);
    assert_eq!(stats.num_channels(), 0);

    // Create 1 channel.
    let (mut sender, _receiver) = client1.create_channel::<()>().claim_sender().await.unwrap();
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent(), 1);
    assert_eq!(stats.messages_received(), 1);
    assert_eq!(stats.num_channels(), 1);

    // Create 2 channels and close 1.
    sender.close().await.unwrap();
    let (sender1, receiver1) = client1.create_channel::<()>().claim_sender().await.unwrap();
    let (_sender2, mut receiver2) = client1
        .create_channel::<()>()
        .claim_receiver(1)
        .await
        .unwrap();
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent(), 3);
    assert_eq!(stats.messages_received(), 3);
    assert_eq!(stats.num_channels(), 2);

    // Claim 1 and send 3 items.
    let mut receiver1 = receiver1.claim(16).await.unwrap();
    let mut sender1 = sender1.establish().await.unwrap();
    sender1.send_item(()).await.unwrap();
    sender1.send_item(()).await.unwrap();
    sender1.send_item(()).await.unwrap();
    client1.sync_broker().await.unwrap();
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent(), 6);
    assert_eq!(stats.messages_received(), 5);
    assert_eq!(stats.num_channels(), 2);

    // Close 2 channels.
    sender1.close().await.unwrap();
    receiver1.close().await.unwrap();
    receiver2.close().await.unwrap();
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent(), 4);
    assert_eq!(stats.messages_received(), 3);
    assert_eq!(stats.num_channels(), 0);

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
    assert_eq!(stats.messages_sent(), 0);
    assert_eq!(stats.messages_received(), 0);
    assert_eq!(stats.num_bus_listeners(), 0);

    // Create 2 bus listeners.
    let mut bus_listener1 = client.create_bus_listener().await.unwrap();
    let mut bus_listener2 = client.create_bus_listener().await.unwrap();
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent(), 2);
    assert_eq!(stats.messages_received(), 2);
    assert_eq!(stats.num_bus_listeners(), 2);

    // Destroy 1 bus listener.
    bus_listener1.destroy().await.unwrap();
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent(), 1);
    assert_eq!(stats.messages_received(), 1);
    assert_eq!(stats.num_bus_listeners(), 1);

    // Destroy 1 bus listener.
    bus_listener2.destroy().await.unwrap();
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.messages_sent(), 1);
    assert_eq!(stats.messages_received(), 1);
    assert_eq!(stats.num_bus_listeners(), 0);

    client.join().await;
    broker.join().await;
}
