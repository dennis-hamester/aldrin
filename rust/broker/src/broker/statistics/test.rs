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
async fn num_connections() {
    let mut broker = TestBroker::new();

    assert_eq!(broker.take_statistics().await.unwrap().num_connections, 0);

    let mut client1 = broker.add_client().await;
    assert_eq!(broker.take_statistics().await.unwrap().num_connections, 1);

    let mut client2 = broker.add_client().await;
    assert_eq!(broker.take_statistics().await.unwrap().num_connections, 2);

    client1.join().await;
    assert_eq!(broker.take_statistics().await.unwrap().num_connections, 1);

    client2.join().await;
    assert_eq!(broker.take_statistics().await.unwrap().num_connections, 0);

    broker.join().await;
}
