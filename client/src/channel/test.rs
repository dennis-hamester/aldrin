use aldrin_test::aldrin_client::Error;
use aldrin_test::tokio_based::TestBroker;
use futures::stream::FusedStream;
use std::time::Duration;
use tokio::time;

#[tokio::test]
async fn create_and_close() {
    let mut broker = TestBroker::new();
    let mut client = broker.add_client().await;

    // PendingSender & UnclaimedReceiver
    let (mut sender, mut receiver) = client
        .create_channel_with_claimed_sender::<()>()
        .await
        .unwrap();
    assert_eq!(sender.close().await, Ok(())); // This also closes the unclaimed receiver.
    assert_eq!(sender.close().await, Ok(()));
    assert_eq!(receiver.close().await, Err(Error::InvalidChannel));
    assert_eq!(receiver.close().await, Ok(()));

    // PendingSender & UnclaimedReceiver
    let (mut sender, mut receiver) = client
        .create_channel_with_claimed_sender::<()>()
        .await
        .unwrap();
    assert_eq!(receiver.close().await, Ok(()));
    assert_eq!(receiver.close().await, Ok(()));
    assert_eq!(sender.close().await, Ok(()));
    assert_eq!(sender.close().await, Ok(()));

    // UnclaimedSender & PendingReceiver
    let (mut sender, mut receiver) = client
        .create_channel_with_claimed_receiver::<()>(1)
        .await
        .unwrap();
    assert_eq!(receiver.close().await, Ok(())); // This also closes the unclaimed sender.
    assert_eq!(receiver.close().await, Ok(()));
    assert_eq!(sender.close().await, Err(Error::InvalidChannel));
    assert_eq!(sender.close().await, Ok(()));

    // UnclaimedSender & PendingReceiver
    let (mut sender, mut receiver) = client
        .create_channel_with_claimed_receiver::<()>(1)
        .await
        .unwrap();
    assert_eq!(sender.close().await, Ok(()));
    assert_eq!(sender.close().await, Ok(()));
    assert_eq!(receiver.close().await, Ok(()));
    assert_eq!(receiver.close().await, Ok(()));

    // PendingSender & Receiver
    let (mut sender, receiver) = client
        .create_channel_with_claimed_sender::<()>()
        .await
        .unwrap();
    let mut receiver = receiver.claim(16).await.unwrap();
    assert_eq!(sender.close().await, Ok(()));
    assert_eq!(sender.close().await, Ok(()));
    assert_eq!(receiver.close().await, Ok(()));
    assert_eq!(receiver.close().await, Ok(()));

    // PendingSender & Receiver
    let (mut sender, receiver) = client
        .create_channel_with_claimed_sender::<()>()
        .await
        .unwrap();
    let mut receiver = receiver.claim(16).await.unwrap();
    assert_eq!(receiver.close().await, Ok(()));
    assert_eq!(receiver.close().await, Ok(()));
    assert_eq!(sender.close().await, Ok(()));
    assert_eq!(sender.close().await, Ok(()));

    // Sender & PendingReceiver
    let (sender, mut receiver) = client
        .create_channel_with_claimed_receiver::<()>(1)
        .await
        .unwrap();
    let mut sender = sender.claim().await.unwrap();
    assert_eq!(sender.close().await, Ok(()));
    assert_eq!(sender.close().await, Ok(()));
    assert_eq!(receiver.close().await, Ok(()));
    assert_eq!(receiver.close().await, Ok(()));

    // Sender & PendingReceiver
    let (sender, mut receiver) = client
        .create_channel_with_claimed_receiver::<()>(1)
        .await
        .unwrap();
    let mut sender = sender.claim().await.unwrap();
    assert_eq!(receiver.close().await, Ok(()));
    assert_eq!(receiver.close().await, Ok(()));
    assert_eq!(sender.close().await, Ok(()));
    assert_eq!(sender.close().await, Ok(()));

    // Sender & Receiver
    let (sender, receiver) = client
        .create_channel_with_claimed_sender::<()>()
        .await
        .unwrap();
    let mut receiver = receiver.claim(16).await.unwrap();
    let mut sender = sender.established().await.unwrap();
    assert_eq!(sender.close().await, Ok(()));
    assert_eq!(sender.close().await, Ok(()));
    assert_eq!(receiver.close().await, Ok(()));
    assert_eq!(receiver.close().await, Ok(()));

    // Sender & Receiver
    let (sender, receiver) = client
        .create_channel_with_claimed_sender::<()>()
        .await
        .unwrap();
    let mut receiver = receiver.claim(16).await.unwrap();
    let mut sender = sender.established().await.unwrap();
    assert_eq!(receiver.close().await, Ok(()));
    assert_eq!(receiver.close().await, Ok(()));
    assert_eq!(sender.close().await, Ok(()));
    assert_eq!(sender.close().await, Ok(()));

    // Sender & Receiver
    let (sender, receiver) = client
        .create_channel_with_claimed_receiver::<()>(1)
        .await
        .unwrap();
    let mut sender = sender.claim().await.unwrap();
    let mut receiver = receiver.established().await.unwrap();
    assert_eq!(sender.close().await, Ok(()));
    assert_eq!(sender.close().await, Ok(()));
    assert_eq!(receiver.close().await, Ok(()));
    assert_eq!(receiver.close().await, Ok(()));

    // Sender & Receiver
    let (sender, receiver) = client
        .create_channel_with_claimed_receiver::<()>(1)
        .await
        .unwrap();
    let mut sender = sender.claim().await.unwrap();
    let mut receiver = receiver.established().await.unwrap();
    assert_eq!(receiver.close().await, Ok(()));
    assert_eq!(receiver.close().await, Ok(()));
    assert_eq!(sender.close().await, Ok(()));
    assert_eq!(sender.close().await, Ok(()));

    client.join().await;
    broker.join().await;
}

#[tokio::test]
async fn send_and_receive() {
    let mut broker = TestBroker::new();
    let mut client = broker.add_client().await;

    let (sender, receiver) = client.create_channel_with_claimed_sender().await.unwrap();

    let mut receiver = receiver.claim(16).await.unwrap();
    let mut sender = sender.established().await.unwrap();

    sender.send_item(&1).await.unwrap();
    assert_eq!(receiver.next_item().await, Ok(Some(1)));

    sender.send_item(&2).await.unwrap();
    sender.send_item(&3).await.unwrap();
    assert_eq!(receiver.next_item().await, Ok(Some(2)));
    assert_eq!(receiver.next_item().await, Ok(Some(3)));

    sender.send_item(&4).await.unwrap();
    sender.send_item(&5).await.unwrap();
    sender.close().await.unwrap();
    assert_eq!(receiver.next_item().await, Ok(Some(4)));
    assert_eq!(receiver.next_item().await, Ok(Some(5)));
    assert_eq!(receiver.next_item().await, Ok(None));
    assert!(receiver.is_terminated());

    receiver.close().await.unwrap();

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
        .claim(client2.handle.clone(), 16)
        .await
        .unwrap();
    let mut sender = sender.established().await.unwrap();

    sender.send_item(&"hello".to_owned()).await.unwrap();
    assert_eq!(receiver.next_item().await, Ok(Some("hello".to_string())));

    client1.join().await;
    client2.join().await;
    broker.join().await;
}

#[tokio::test]
async fn send_error_when_receiver_is_closed() {
    let mut broker = TestBroker::new();
    let mut client1 = broker.add_client().await;
    let mut client2 = broker.add_client().await;

    let (sender, receiver) = client1
        .create_channel_with_claimed_sender::<u32>()
        .await
        .unwrap();

    let mut receiver = receiver.unbind().claim(client2.clone(), 16).await.unwrap();
    let mut sender = sender.established().await.unwrap();

    receiver.close().await.unwrap();

    let timeout = time::sleep(Duration::from_millis(500));
    tokio::pin!(timeout);
    let mut interval = time::interval(Duration::from_millis(50));
    loop {
        tokio::select! {
            () = &mut timeout => panic!("timeout reached"),
            _ = interval.tick() => {
                if sender.send_item(&0).await.is_err() {
                    break;
                }
            }
        }
    }

    client1.join().await;
    client2.join().await;
    broker.join().await;
}
