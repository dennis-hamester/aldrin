use std::mem;
use std::time::Duration;
use tokio::time;

#[tokio::test]
async fn client_stops_when_last_handle_is_dropped() {
    let mut broker = aldrin_test::TestBroker::new();
    tokio::spawn(broker.take_broker().run());

    let mut client = broker.add_client().await;
    tokio::spawn(client.take_connection().run());
    let join = tokio::spawn(client.take_client().run());

    let _ = client.clone();
    let _ = client.clone();
    let _ = client.clone();
    let _ = client.clone();
    mem::drop(client);

    time::timeout(Duration::from_millis(100), join)
        .await
        .unwrap()
        .unwrap()
        .unwrap();
}
