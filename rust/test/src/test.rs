use super::TestBroker;
use std::time::Duration;
use tokio::time;

#[tokio::test]
async fn shutdown_idle() {
    let test = async {
        let mut broker = TestBroker::new();
        let broker_join = tokio::spawn(broker.take_broker().run());

        let mut client1 = broker.add_client().await;
        let client1_join = tokio::spawn(client1.take_client().run());
        let conn1_join = tokio::spawn(client1.take_connection().run());

        let mut client2 = broker.add_client().await;
        let client2_join = tokio::spawn(client2.take_client().run());
        let conn2_join = tokio::spawn(client2.take_connection().run());

        broker.shutdown_idle().await;

        client1.shutdown();
        client1_join.await.unwrap().unwrap();
        conn1_join.await.unwrap().unwrap();

        client2.shutdown();
        client2_join.await.unwrap().unwrap();
        conn2_join.await.unwrap().unwrap();

        broker_join.await.unwrap();
    };

    time::timeout(Duration::from_secs(1), test).await.unwrap();
}
