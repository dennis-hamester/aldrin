use aldrin_broker::{Broker, BrokerHandle, ConnectionHandle};
use aldrin_client::Client;
use aldrin_client::Handle as ClientHandle;
use aldrin_util::channel::channel;
use anyhow::Result;
use std::num::NonZeroUsize;
use tokio::task::JoinHandle;

pub fn create_broker(fifo_size: Option<usize>) -> (BrokerHandle, JoinHandle<Result<()>>) {
    let broker = if let Some(fifo_size) = fifo_size {
        Broker::with_fifo_size(NonZeroUsize::new(fifo_size))
    } else {
        Broker::new()
    };
    let handle = broker.handle().clone();
    let join = tokio::spawn(async { broker.run().await.map_err(Into::into) });
    (handle, join)
}

pub async fn create_client(
    broker_handle: BrokerHandle,
    channel_fifo_size: usize,
    conn_fifo_size: Option<usize>,
) -> Result<(
    ClientHandle,
    JoinHandle<Result<()>>,
    ConnectionHandle,
    JoinHandle<Result<()>>,
)> {
    let (t1, t2) = channel(channel_fifo_size);

    let add_conn_join = if let Some(conn_fifo_size) = conn_fifo_size {
        tokio::spawn(async move {
            broker_handle
                .add_connection_with_fifo_size(t1, NonZeroUsize::new(conn_fifo_size))
                .await
        })
    } else {
        tokio::spawn(async move { broker_handle.add_connection(t1).await })
    };

    let client = Client::connect(t2).await?;
    let client_handle = client.handle().clone();
    let client_run_join = tokio::spawn(async move { client.run().await.map_err(Into::into) });

    let conn = add_conn_join.await??;
    let conn_handle = conn.handle().clone();
    let conn_run_join = tokio::spawn(async move { conn.run().await.map_err(Into::into) });

    Ok((client_handle, client_run_join, conn_handle, conn_run_join))
}
