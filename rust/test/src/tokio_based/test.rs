use super::TestBroker;
use std::time::Duration;
use tokio::time;

#[tokio::test]
async fn shutdown_idle() {
    let test = async {
        let mut broker = TestBroker::new();
        let mut client1 = broker.add_client().await;
        let mut client2 = broker.add_client().await;

        client1.join().await;
        client2.join().await;
        broker.join_idle().await;
    };

    time::timeout(Duration::from_secs(1), test).await.unwrap();
}

#[tokio::test]
async fn client_builder() {
    let broker = TestBroker::new();
    let mut builder = broker.client_builder();
    let mut base = builder.0.clone();

    assert_eq!(builder.0.channel, base.channel);

    builder = builder.bounded_channel(1);
    base = base.bounded_channel(1);
    assert_eq!(builder.0.channel, base.channel);

    builder = builder.unbounded_channel();
    base = base.unbounded_channel();
    assert_eq!(builder.0.channel, base.channel);
}
