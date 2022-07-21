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
    assert_eq!(stats.num_connections, 0);
    assert_eq!(stats.connections_added, 0);

    // Add 1 client.
    let mut client1 = broker.add_client().await;
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.num_connections, 1);
    assert_eq!(stats.connections_added, 1);

    // Remove 1 client and add 2.
    client1.join().await;
    let mut client2 = broker.add_client().await;
    let mut client3 = broker.add_client().await;
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.num_connections, 2);
    assert_eq!(stats.connections_added, 2);

    // Remove 2 clients.
    client2.join().await;
    client3.join().await;
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.num_connections, 0);
    assert_eq!(stats.connections_added, 0);

    broker.join().await;
}
