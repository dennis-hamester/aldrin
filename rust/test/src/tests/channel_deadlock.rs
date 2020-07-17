use crate::util;
use aldrin_client::{ObjectUuid, SubscribeMode};
use anyhow::Result;
use std::time::Duration;
use tokio::time::timeout;

const FIFO_SIZE: usize = 4;

// Test that a misbehaving client which doesn't drain all of its messages doesn't deadlock all the
// way to the broker, effectively deadlocking the whole bus.
#[tokio::test]
async fn channel_deadlock() -> Result<()> {
    let (broker, broker_join) = util::create_broker(Some(FIFO_SIZE));
    let (client, client_join, _, conn_join) =
        util::create_client(broker.clone(), FIFO_SIZE, Some(FIFO_SIZE)).await?;

    // This is never drained.
    let _objs = client.objects(SubscribeMode::All)?;

    timeout(Duration::from_secs(1), async move {
        for _ in 0..32 {
            client.create_object(ObjectUuid::new_v4()).await?;
        }

        broker.shutdown();
        broker_join.await??;
        client_join.await??;
        conn_join.await??;

        Ok(())
    })
    .await?
}
