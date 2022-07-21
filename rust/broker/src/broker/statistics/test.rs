use aldrin_client::{ObjectUuid, ServiceUuid};
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
