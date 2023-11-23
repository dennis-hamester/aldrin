use aldrin_test::tokio::TestBroker;

#[tokio::test]
async fn create_and_explicit_destroy() {
    let mut broker = TestBroker::new();
    let mut client = broker.add_client().await;

    let mut bus_listener = client.create_bus_listener().await.unwrap();
    bus_listener.destroy().await.unwrap();
    assert!(bus_listener.destroy().await.is_err());

    client.join().await;
    broker.join().await;
}

#[tokio::test]
async fn create_and_implicit_destroy() {
    let mut broker = TestBroker::new();
    let mut client = broker.add_client().await;

    client.create_bus_listener().await.unwrap();

    client.join().await;
    broker.join().await;
}
