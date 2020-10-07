use aldrin_test::aldrin_client::ObjectUuid;
use aldrin_test::tokio_based::TestBroker;
use std::future::Future;
use std::mem;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::time;

struct PollOnce<Fut>(Fut);

impl<Fut: Future + Unpin> Future for PollOnce<Fut> {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let _ = Pin::new(&mut self.0).poll(cx);
        Poll::Ready(())
    }
}

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

#[tokio::test]
async fn abort_create_object() {
    let broker = TestBroker::new();
    let client = broker.add_client().await;

    let uuid = ObjectUuid::new_v4();
    let fut = client.create_object(uuid);

    // This assumes that polling the future once is enough to create the object.
    PollOnce(Box::pin(fut)).await;

    // The object may have been created temporarily. Give client and broker some time to destroy it
    // again.
    time::delay_for(Duration::from_millis(100)).await;

    assert!(client.resolve_object(uuid).await.unwrap().is_none());
}
