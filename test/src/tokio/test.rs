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
