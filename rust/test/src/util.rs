use aldrin_broker::{Broker, BrokerHandle, ConnectionHandle};
use aldrin_client::Client;
use aldrin_client::Handle as ClientHandle;
use aldrin_util::channel::channel;
use anyhow::Result;
use tokio::task::JoinHandle;

pub fn create_broker(fifo_size: usize) -> (BrokerHandle, JoinHandle<()>) {
    let broker = Broker::new(fifo_size);
    let handle = broker.handle().clone();
    let join = tokio::spawn(broker.run());
    (handle, join)
}

pub async fn create_client(
    mut broker_handle: BrokerHandle,
    channel_fifo_size: usize,
    client_fifo_size: usize,
    conn_fifo_size: usize,
) -> Result<(
    ClientHandle,
    JoinHandle<Result<()>>,
    ConnectionHandle,
    JoinHandle<Result<()>>,
)> {
    let (t1, t2) = channel(channel_fifo_size);

    let add_conn_join =
        tokio::spawn(async move { broker_handle.add_connection(t1, conn_fifo_size).await });

    let client = Client::connect(t2, client_fifo_size).await?;
    let client_handle = client.handle().clone();
    let client_run_join = tokio::spawn(async move { client.run().await.map_err(Into::into) });

    let conn = add_conn_join.await??;
    let conn_handle = conn.handle().clone();
    let conn_run_join = tokio::spawn(async move { conn.run().await.map_err(Into::into) });

    Ok((client_handle, client_run_join, conn_handle, conn_run_join))
}
