use aldrin_test::aldrin::Error;
use aldrin_test::tokio::TestBroker;
use futures_util::stream::FusedStream;
use std::mem;
use std::time::Duration;
use tokio::time;

#[tokio::test]
async fn create_and_close() {
    let mut broker = TestBroker::new();
    let mut client = broker.add_client().await;

    // PendingSender & UnclaimedReceiver
    let (mut sender, mut receiver) = client.create_channel::<()>().claim_sender().await.unwrap();
    assert_eq!(sender.close().await, Ok(())); // This also closes the unclaimed receiver.
    assert_eq!(sender.close().await, Ok(()));
    assert_eq!(receiver.close().await, Err(Error::InvalidChannel));
    assert_eq!(receiver.close().await, Ok(()));

    // PendingSender & UnclaimedReceiver
    let (mut sender, mut receiver) = client.create_channel::<()>().claim_sender().await.unwrap();
    assert_eq!(receiver.close().await, Ok(()));
    assert_eq!(receiver.close().await, Ok(()));
    assert_eq!(sender.close().await, Ok(()));
    assert_eq!(sender.close().await, Ok(()));

    // UnclaimedSender & PendingReceiver
    let (mut sender, mut receiver) = client
        .create_channel::<()>()
        .claim_receiver(16)
        .await
        .unwrap();
    assert_eq!(receiver.close().await, Ok(())); // This also closes the unclaimed sender.
    assert_eq!(receiver.close().await, Ok(()));
    assert_eq!(sender.close().await, Err(Error::InvalidChannel));
    assert_eq!(sender.close().await, Ok(()));

    // UnclaimedSender & PendingReceiver
    let (mut sender, mut receiver) = client
        .create_channel::<()>()
        .claim_receiver(1)
        .await
        .unwrap();
    assert_eq!(sender.close().await, Ok(()));
    assert_eq!(sender.close().await, Ok(()));
    assert_eq!(receiver.close().await, Ok(()));
    assert_eq!(receiver.close().await, Ok(()));

    // PendingSender & Receiver
    let (mut sender, receiver) = client.create_channel::<()>().claim_sender().await.unwrap();
    let mut receiver = receiver.claim(16).await.unwrap();
    assert_eq!(sender.close().await, Ok(()));
    assert_eq!(sender.close().await, Ok(()));
    assert_eq!(receiver.close().await, Ok(()));
    assert_eq!(receiver.close().await, Ok(()));

    // PendingSender & Receiver
    let (mut sender, receiver) = client.create_channel::<()>().claim_sender().await.unwrap();
    let mut receiver = receiver.claim(16).await.unwrap();
    assert_eq!(receiver.close().await, Ok(()));
    assert_eq!(receiver.close().await, Ok(()));
    assert_eq!(sender.close().await, Ok(()));
    assert_eq!(sender.close().await, Ok(()));

    // Sender & PendingReceiver
    let (sender, mut receiver) = client
        .create_channel::<()>()
        .claim_receiver(1)
        .await
        .unwrap();
    let mut sender = sender.claim().await.unwrap();
    assert_eq!(sender.close().await, Ok(()));
    assert_eq!(sender.close().await, Ok(()));
    assert_eq!(receiver.close().await, Ok(()));
    assert_eq!(receiver.close().await, Ok(()));

    // Sender & PendingReceiver
    let (sender, mut receiver) = client
        .create_channel::<()>()
        .claim_receiver(1)
        .await
        .unwrap();
    let mut sender = sender.claim().await.unwrap();
    assert_eq!(receiver.close().await, Ok(()));
    assert_eq!(receiver.close().await, Ok(()));
    assert_eq!(sender.close().await, Ok(()));
    assert_eq!(sender.close().await, Ok(()));

    // Sender & Receiver
    let (sender, receiver) = client.create_channel::<()>().claim_sender().await.unwrap();
    let mut receiver = receiver.claim(16).await.unwrap();
    let mut sender = sender.establish().await.unwrap();
    assert_eq!(sender.close().await, Ok(()));
    assert_eq!(sender.close().await, Ok(()));
    assert_eq!(receiver.close().await, Ok(()));
    assert_eq!(receiver.close().await, Ok(()));

    // Sender & Receiver
    let (sender, receiver) = client.create_channel::<()>().claim_sender().await.unwrap();
    let mut receiver = receiver.claim(16).await.unwrap();
    let mut sender = sender.establish().await.unwrap();
    assert_eq!(receiver.close().await, Ok(()));
    assert_eq!(receiver.close().await, Ok(()));
    assert_eq!(sender.close().await, Ok(()));
    assert_eq!(sender.close().await, Ok(()));

    // Sender & Receiver
    let (sender, receiver) = client
        .create_channel::<()>()
        .claim_receiver(1)
        .await
        .unwrap();
    let mut sender = sender.claim().await.unwrap();
    let mut receiver = receiver.establish().await.unwrap();
    assert_eq!(sender.close().await, Ok(()));
    assert_eq!(sender.close().await, Ok(()));
    assert_eq!(receiver.close().await, Ok(()));
    assert_eq!(receiver.close().await, Ok(()));

    // Sender & Receiver
    let (sender, receiver) = client
        .create_channel::<()>()
        .claim_receiver(1)
        .await
        .unwrap();
    let mut sender = sender.claim().await.unwrap();
    let mut receiver = receiver.establish().await.unwrap();
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

    let (sender, receiver) = client.create_channel().claim_sender().await.unwrap();

    let mut receiver = receiver.claim(16).await.unwrap();
    let mut sender = sender.establish().await.unwrap();

    sender.send_item(1).await.unwrap();
    assert_eq!(receiver.next_item().await, Ok(Some(1)));

    sender.send_item(2).await.unwrap();
    sender.send_item(3).await.unwrap();
    assert_eq!(receiver.next_item().await, Ok(Some(2)));
    assert_eq!(receiver.next_item().await, Ok(Some(3)));

    sender.send_item(4).await.unwrap();
    sender.send_item(5).await.unwrap();
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
        .create_channel::<String>()
        .claim_sender()
        .await
        .unwrap();

    let mut receiver = receiver.unbind().claim(client2.clone(), 16).await.unwrap();
    let mut sender = sender.establish().await.unwrap();

    sender.send_item("hello").await.unwrap();
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
        .create_channel::<u32>()
        .claim_sender()
        .await
        .unwrap();

    let mut receiver = receiver.unbind().claim(client2.clone(), 16).await.unwrap();
    let mut sender = sender.establish().await.unwrap();

    receiver.close().await.unwrap();

    let timeout = time::sleep(Duration::from_millis(500));
    tokio::pin!(timeout);
    let mut interval = time::interval(Duration::from_millis(50));
    loop {
        tokio::select! {
            () = &mut timeout => panic!("timeout reached"),
            _ = interval.tick() => {
                if sender.send_item(0).await.is_err() {
                    break;
                }
            }
        }
    }

    client1.join().await;
    client2.join().await;
    broker.join().await;
}

#[cfg(feature = "sink")]
#[tokio::test]
async fn stream_sink_pipe() {
    use aldrin_test::aldrin::{Handle, Receiver, Sender};
    use futures_util::{stream, SinkExt, TryStreamExt};

    async fn create_channel(client: &Handle, capacity: u32) -> (Sender<u32>, Receiver<u32>) {
        let (sender, receiver) = client.create_channel().claim_sender().await.unwrap();
        let receiver = receiver.claim(capacity).await.unwrap();
        let sender = sender.establish().await.unwrap();
        (sender, receiver)
    }

    let mut broker = TestBroker::new();
    let mut client1 = broker.add_client().await;
    let mut client2 = broker.add_client().await;

    let (mut sender1, mut receiver1) = create_channel(&client1, 3).await;
    let (mut sender2, receiver2) = create_channel(&client2, 7).await;

    // Pipe receiver1 into sender2, so that items sent via sender1 are received via receiver2.
    let pipe_join = tokio::spawn(async move { sender2.send_all(&mut receiver1).await });
    let items_received = tokio::spawn(async move { receiver2.try_collect::<Vec<_>>().await });

    let items_sent: Vec<_> = (0..128).collect();

    sender1
        .send_all(&mut stream::iter(items_sent.iter().copied().map(Ok)))
        .await
        .unwrap();

    // Closing sender1 will lead to this in order:
    // - Close receiver1.
    // - Cause the task of pipe_join to finish.
    // - Close sender2.
    // - Close receiver2.
    // - Cause the task of items_received to finish.
    sender1.close().await.unwrap();

    pipe_join.await.unwrap().unwrap();
    let items_received = items_received.await.unwrap().unwrap();

    assert_eq!(items_sent, items_received);

    client1.join().await;
    client2.join().await;
    broker.join().await;
}

#[tokio::test]
async fn sender_closed_implicit() {
    let mut broker = TestBroker::new();
    let mut client = broker.add_client().await;

    let (sender, receiver) = client.create_channel::<u32>().claim_sender().await.unwrap();

    let receiver = receiver.claim(16).await.unwrap();
    let mut sender = sender.establish().await.unwrap();

    mem::drop(receiver);
    sender.receiver_closed().await;

    client.join().await;
    broker.join().await;
}

#[tokio::test]
async fn not_leaking_pending_senders() {
    let mut broker = TestBroker::new();
    let mut client = broker.add_client().await;

    let (sender, receiver) = client.create_channel::<()>().claim_sender().await.unwrap();

    // Dropping the receiver will close that half and establishing the sender will fail.
    mem::drop(receiver);
    assert_eq!(sender.establish().await.unwrap_err(), Error::InvalidChannel);

    client.sync_broker().await.unwrap();
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.num_channels(), 0);

    client.join().await;
    broker.join().await;
}

#[tokio::test]
async fn not_leaking_pending_receivers() {
    let mut broker = TestBroker::new();
    let mut client = broker.add_client().await;

    let (sender, receiver) = client
        .create_channel::<()>()
        .claim_receiver(1)
        .await
        .unwrap();

    // Dropping the sender will close that half and establishing the receiver will fail.
    mem::drop(sender);
    assert_eq!(
        receiver.establish().await.unwrap_err(),
        Error::InvalidChannel
    );

    client.sync_broker().await.unwrap();
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.num_channels(), 0);

    client.join().await;
    broker.join().await;
}

#[tokio::test]
async fn not_leaking_senders() {
    let mut broker = TestBroker::new();
    let mut client = broker.add_client().await;

    let (sender, receiver) = client.create_channel::<()>().claim_sender().await.unwrap();
    let receiver = receiver.claim(16).await.unwrap();
    let mut sender = sender.establish().await.unwrap();

    // Dropping the receiver will close that half and notify the sender.
    mem::drop(receiver);
    sender.receiver_closed().await;

    // Dropping the sender will close the other half and then the entire channel.
    mem::drop(sender);

    client.sync_broker().await.unwrap();
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.num_channels(), 0);

    client.join().await;
    broker.join().await;
}

#[tokio::test]
async fn not_leaking_receivers() {
    let mut broker = TestBroker::new();
    let mut client = broker.add_client().await;

    let (sender, receiver) = client.create_channel::<()>().claim_sender().await.unwrap();
    let mut receiver = receiver.claim(16).await.unwrap();
    let sender = sender.establish().await.unwrap();

    // Dropping the sender will close that half and notify the receiver.
    mem::drop(sender);
    assert_eq!(receiver.next_item().await, Ok(None));

    // Dropping the receiver will close the other half and then the entire channel.
    mem::drop(receiver);

    client.sync_broker().await.unwrap();
    let stats = broker.take_statistics().await.unwrap();
    assert_eq!(stats.num_channels(), 0);

    client.join().await;
    broker.join().await;
}
