use aldrin_proto::Value;
use aldrin_test::aldrin_client::Error;
use aldrin_test::tokio_based::TestBroker;
use futures::stream::{FusedStream, StreamExt};
use std::time::Duration;
use tokio::time;

#[tokio::test]
async fn create_and_destroy() {
    let mut broker = TestBroker::new();
    let mut client = broker.add_client().await;

    // PendingSender & UnclaimedReceiver
    let (mut sender, mut receiver) = client
        .create_channel_with_claimed_sender::<()>()
        .await
        .unwrap();
    assert_eq!(sender.destroy().await, Ok(())); // This also destroys the unclaimed receiver.
    assert_eq!(receiver.destroy().await, Err(Error::InvalidChannel));

    // PendingSender & UnclaimedReceiver
    let (mut sender, mut receiver) = client
        .create_channel_with_claimed_sender::<()>()
        .await
        .unwrap();
    assert_eq!(receiver.destroy().await, Ok(()));
    assert_eq!(sender.destroy().await, Ok(()));

    // UnclaimedSender & PendingReceiver
    let (mut sender, mut receiver) = client
        .create_channel_with_claimed_receiver::<()>()
        .await
        .unwrap();
    assert_eq!(receiver.destroy().await, Ok(())); // This also destroys the unclaimed sender.
    assert_eq!(sender.destroy().await, Err(Error::InvalidChannel));

    // UnclaimedSender & PendingReceiver
    let (mut sender, mut receiver) = client
        .create_channel_with_claimed_receiver::<()>()
        .await
        .unwrap();
    assert_eq!(sender.destroy().await, Ok(()));
    assert_eq!(receiver.destroy().await, Ok(()));

    // PendingSender & Receiver
    let (mut sender, receiver) = client
        .create_channel_with_claimed_sender::<()>()
        .await
        .unwrap();
    let mut receiver = receiver.claim().await.unwrap();
    assert_eq!(sender.destroy().await, Ok(()));
    assert_eq!(receiver.destroy().await, Ok(()));

    // PendingSender & Receiver
    let (mut sender, receiver) = client
        .create_channel_with_claimed_sender::<()>()
        .await
        .unwrap();
    let mut receiver = receiver.claim().await.unwrap();
    assert_eq!(receiver.destroy().await, Ok(()));
    assert_eq!(sender.destroy().await, Ok(()));

    // Sender & PendingReceiver
    let (sender, mut receiver) = client
        .create_channel_with_claimed_receiver::<()>()
        .await
        .unwrap();
    let mut sender = sender.claim().await.unwrap();
    assert_eq!(sender.destroy().await, Ok(()));
    assert_eq!(receiver.destroy().await, Ok(()));

    // Sender & PendingReceiver
    let (sender, mut receiver) = client
        .create_channel_with_claimed_receiver::<()>()
        .await
        .unwrap();
    let mut sender = sender.claim().await.unwrap();
    assert_eq!(receiver.destroy().await, Ok(()));
    assert_eq!(sender.destroy().await, Ok(()));

    // Sender & Receiver
    let (sender, receiver) = client
        .create_channel_with_claimed_sender::<()>()
        .await
        .unwrap();
    let mut receiver = receiver.claim().await.unwrap();
    let mut sender = sender.established().await.unwrap();
    assert_eq!(sender.destroy().await, Ok(()));
    assert_eq!(receiver.destroy().await, Ok(()));

    // Sender & Receiver
    let (sender, receiver) = client
        .create_channel_with_claimed_sender::<()>()
        .await
        .unwrap();
    let mut receiver = receiver.claim().await.unwrap();
    let mut sender = sender.established().await.unwrap();
    assert_eq!(receiver.destroy().await, Ok(()));
    assert_eq!(sender.destroy().await, Ok(()));

    // Sender & Receiver
    let (sender, receiver) = client
        .create_channel_with_claimed_receiver::<()>()
        .await
        .unwrap();
    let mut sender = sender.claim().await.unwrap();
    let mut receiver = receiver.established().await.unwrap();
    assert_eq!(sender.destroy().await, Ok(()));
    assert_eq!(receiver.destroy().await, Ok(()));

    // Sender & Receiver
    let (sender, receiver) = client
        .create_channel_with_claimed_receiver::<()>()
        .await
        .unwrap();
    let mut sender = sender.claim().await.unwrap();
    let mut receiver = receiver.established().await.unwrap();
    assert_eq!(receiver.destroy().await, Ok(()));
    assert_eq!(sender.destroy().await, Ok(()));

    client.join().await;
    broker.join().await;
}

#[tokio::test]
async fn send_and_receive() {
    let mut broker = TestBroker::new();
    let mut client = broker.add_client().await;

    let (sender, receiver) = client
        .create_channel_with_claimed_sender::<Value>()
        .await
        .unwrap();

    let mut receiver = receiver.claim().await.unwrap();
    let mut sender = sender.established().await.unwrap();

    sender.send(Value::U32(1)).unwrap();
    assert_eq!(receiver.next().await, Some(Ok(Value::U32(1))));

    sender.send(Value::U32(2)).unwrap();
    sender.send(Value::U32(3)).unwrap();
    assert_eq!(receiver.next().await, Some(Ok(Value::U32(2))));
    assert_eq!(receiver.next().await, Some(Ok(Value::U32(3))));

    sender.send(Value::U32(4)).unwrap();
    sender.send(Value::U32(5)).unwrap();
    sender.destroy().await.unwrap();
    assert_eq!(receiver.next().await, Some(Ok(Value::U32(4))));
    assert_eq!(receiver.next().await, Some(Ok(Value::U32(5))));
    assert_eq!(receiver.next().await, None);
    assert!(receiver.is_terminated());

    receiver.destroy().await.unwrap();

    client.join().await;
    broker.join().await;
}

#[tokio::test]
async fn multiple_clients() {
    let mut broker = TestBroker::new();
    let mut client1 = broker.add_client().await;
    let mut client2 = broker.add_client().await;

    let (sender, receiver) = client1
        .create_channel_with_claimed_sender::<String>()
        .await
        .unwrap();

    let mut receiver = receiver
        .unbind()
        .claim(client2.handle.clone())
        .await
        .unwrap();
    let mut sender = sender.established().await.unwrap();

    sender.send("hello".to_owned()).unwrap();
    assert_eq!(receiver.next().await.unwrap().as_deref(), Ok("hello"));

    client1.join().await;
    client2.join().await;
    broker.join().await;
}

#[tokio::test]
async fn send_error_when_receiver_is_destroyed() {
    let mut broker = TestBroker::new();
    let mut client1 = broker.add_client().await;
    let mut client2 = broker.add_client().await;

    let (sender, receiver) = client1
        .create_channel_with_claimed_sender::<u32>()
        .await
        .unwrap();

    let mut receiver = receiver.unbind().claim(client2.clone()).await.unwrap();
    let mut sender = sender.established().await.unwrap();

    receiver.destroy().await.unwrap();

    let timeout = time::sleep(Duration::from_millis(500));
    tokio::pin!(timeout);
    let mut interval = time::interval(Duration::from_millis(50));
    loop {
        tokio::select! {
            () = &mut timeout => panic!("timeout reached"),
            _ = interval.tick() => {
                if sender.send(0).is_err() {
                    break;
                }
            }
        }
    }

    client1.join().await;
    client2.join().await;
    broker.join().await;
}
